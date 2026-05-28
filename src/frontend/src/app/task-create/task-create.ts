import { HttpErrorResponse } from '@angular/common/http';
import { CommonModule } from '@angular/common';
import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { catchError, finalize, forkJoin, map, of, Subscription, switchMap } from 'rxjs';

import { AuthApiService, CurrentUser } from '../auth-api.service';
import { Label, LabelsApiService } from '../labels-api.service';
import { ProjectMember, ProjectMembersApiService } from '../project-members-api.service';
import {
  CreatedFeature,
  ProjectApiService,
  ProjectDetail as ProjectDetailModel,
  TASK_PRIORITY_OPTIONS,
} from '../project-api.service';

type PageLoadResult =
  | { kind: 'invalid' }
  | {
      kind: 'ok';
      project: ProjectDetailModel;
      feature: CreatedFeature;
      currentUser: CurrentUser;
      assigneeOptions: Array<{ id: string; label: string }>;
      labels: Label[];
    }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-task-create',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './task-create.html',
  styleUrls: ['../feature-create/feature-create.scss', './task-create.scss'],
})
export class TaskCreate implements OnInit, OnDestroy {
  private readonly fb = inject(FormBuilder);
  private readonly projectApi = inject(ProjectApiService);
  private readonly authApi = inject(AuthApiService);
  private readonly membersApi = inject(ProjectMembersApiService);
  private readonly labelsApi = inject(LabelsApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly project = signal<ProjectDetailModel | null>(null);
  protected readonly feature = signal<CreatedFeature | null>(null);
  protected readonly currentUser = signal<CurrentUser | null>(null);
  protected readonly pageError = signal<string | null>(null);
  protected readonly assigneeOptions = signal<Array<{ id: string; label: string }>>([]);
  protected readonly labels = signal<Label[]>([]);

  protected readonly priorityOptions = [...TASK_PRIORITY_OPTIONS];

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly showSuccess = signal(false);

  protected readonly form = this.fb.nonNullable.group({
    title: ['', [Validators.required, Validators.maxLength(512)]],
    description: [''],
    priority: this.fb.nonNullable.control<string>('Standard', Validators.required),
    dueDate: this.fb.nonNullable.control<string>(''),
    dueTime: this.fb.nonNullable.control<string>(''),
    assigneeUserId: this.fb.nonNullable.control<string>(''),
    labelIds: this.fb.nonNullable.control<string[]>([]),
  });

  ngOnInit(): void {
    this.sub = this.route.paramMap
      .pipe(
        switchMap((params) => {
          const projectId = params.get('projectId') ?? '';
          const featureId = params.get('featureId') ?? '';
          if (projectId.length === 0 || featureId.length === 0) {
            return of<PageLoadResult>({ kind: 'invalid' });
          }
          this.loadState.set('loading');
          this.pageError.set(null);
          return this.authApi.me().pipe(
            switchMap((currentUser) =>
              forkJoin({
                project: this.projectApi.getProject(projectId),
                feature: this.projectApi.getFeature(projectId, featureId),
                members:
                  currentUser.accountType === 'company'
                    ? this.membersApi.listMembers(projectId)
                    : of([] as ProjectMember[]),
                labels: this.labelsApi.listLabels(),
              }).pipe(
                map(
                  (data): PageLoadResult => ({
                    kind: 'ok',
                    project: data.project,
                    feature: data.feature,
                    currentUser,
                    assigneeOptions: this.buildAssigneeOptions(currentUser, data.members),
                    labels: data.labels,
                  }),
                ),
              ),
            ),
            catchError((err: unknown) =>
              of<PageLoadResult>({
                kind: 'error',
                message:
                  err instanceof HttpErrorResponse && err.status === 401
                    ? 'You need to sign in again to create a task.'
                    : ProjectApiService.featureDetailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.pageError.set('Missing project or feature identifier.');
          this.project.set(null);
          this.feature.set(null);
          this.currentUser.set(null);
          this.assigneeOptions.set([]);
          this.labels.set([]);
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.project.set(null);
          this.feature.set(null);
          this.currentUser.set(null);
          this.assigneeOptions.set([]);
          this.labels.set([]);
          this.loadState.set('error');
          return;
        }
        this.project.set(res.project);
        this.feature.set(res.feature);
        this.currentUser.set(res.currentUser);
        this.assigneeOptions.set(res.assigneeOptions);
        this.labels.set(res.labels);
        const hasCurrentAssigneeOption = res.assigneeOptions.some(
          (option) => option.id === res.currentUser.id,
        );
        this.form.controls.assigneeUserId.setValue(
          hasCurrentAssigneeOption ? res.currentUser.id : '',
        );
        this.pageError.set(null);
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  protected submit(): void {
    const p = this.project();
    const f = this.feature();
    const cu = this.currentUser();
    if (!p || !f) return;

    this.serverError.set(null);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting()) {
      return;
    }

    const { title, description, priority, dueDate, dueTime, assigneeUserId } = this.form.getRawValue();
    const labelIds = this.form.controls.labelIds.value;
    const unassigned = assigneeUserId === '';
    const dueDateTrimmed = dueDate.trim();
    const dueTimeTrimmed = dueTime.trim();
    if (dueDateTrimmed.length === 0 && dueTimeTrimmed.length > 0) {
      this.serverError.set('Due time cannot be set without a due date.');
      return;
    }
    if (!unassigned && !cu) {
      this.serverError.set('Could not resolve the current account. Please reload and sign in.');
      return;
    }

    this.submitting.set(true);

    this.projectApi
      .createTask(p.id, f.id, {
        title: title.trim(),
        description: description.trim().length > 0 ? description : undefined,
        priority,
        dueDate: dueDateTrimmed.length > 0 ? dueDateTrimmed : undefined,
        dueTime: dueDateTrimmed.length > 0 && dueTimeTrimmed.length > 0 ? dueTimeTrimmed : undefined,
        unassigned,
        assigneeUserId: !unassigned ? assigneeUserId : undefined,
        labelIds,
      })
      .pipe(finalize(() => this.submitting.set(false)))
      .subscribe({
        next: () => {
          this.showSuccess.set(true);
        },
        error: (err: unknown) => {
          this.serverError.set(ProjectApiService.createTaskErrorMessage(err));
        },
      });
  }

  protected cancel(): void {
    const p = this.project();
    const f = this.feature();
    if (p && f) {
      void this.router.navigateByUrl(
        `/app/projects/${encodeURIComponent(p.id)}/features/${encodeURIComponent(f.id)}`,
      );
    } else if (p) {
      void this.router.navigateByUrl(`/app/projects/${encodeURIComponent(p.id)}`);
    } else {
      void this.router.navigateByUrl('/app/projects');
    }
  }

  protected dismissSuccess(): void {
    const p = this.project();
    const f = this.feature();
    this.showSuccess.set(false);
    if (p && f) {
      void this.router.navigateByUrl(
        `/app/projects/${encodeURIComponent(p.id)}/features/${encodeURIComponent(f.id)}`,
      );
    } else if (p) {
      void this.router.navigateByUrl(`/app/projects/${encodeURIComponent(p.id)}`);
    } else {
      void this.router.navigateByUrl('/app/projects');
    }
  }

  protected titleError(): string {
    const c = this.form.controls.title;
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      return 'Task title is required.';
    }
    if (c.errors['maxlength']) {
      return 'Title is too long.';
    }
    return '';
  }

  protected dueDateMissingForTime(): boolean {
    const dueDate = this.form.controls.dueDate.value.trim();
    const dueTime = this.form.controls.dueTime.value.trim();
    return dueDate.length === 0 && dueTime.length > 0;
  }

  protected toggleLabel(labelId: string, checked: boolean): void {
    const current = this.form.controls.labelIds.value;
    if (checked) {
      if (!current.includes(labelId)) {
        this.form.controls.labelIds.setValue([...current, labelId]);
      }
      return;
    }
    this.form.controls.labelIds.setValue(current.filter((id) => id !== labelId));
  }

  protected hasLabel(labelId: string): boolean {
    return this.form.controls.labelIds.value.includes(labelId);
  }

  protected labelTextColor(hex: string): string {
    const upper = hex.trim().toUpperCase();
    if (!/^#[0-9A-F]{6}$/.test(upper)) {
      return '#111827';
    }
    const r = Number.parseInt(upper.slice(1, 3), 16);
    const g = Number.parseInt(upper.slice(3, 5), 16);
    const b = Number.parseInt(upper.slice(5, 7), 16);
    const luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    return luma > 165 ? '#111827' : '#ffffff';
  }

  private buildAssigneeOptions(
    currentUser: CurrentUser,
    members: ProjectMember[],
  ): Array<{ id: string; label: string }> {
    if (currentUser.accountType !== 'company') {
      return [
        { id: '', label: 'Unassigned' },
        { id: currentUser.id, label: currentUser.fullname.trim() || 'Me' },
      ];
    }

    const options: Array<{ id: string; label: string }> = [{ id: '', label: 'Unassigned' }];
    for (const member of members) {
      options.push({
        id: member.userId,
        label: member.fullname.trim().length > 0 ? member.fullname.trim() : member.email,
      });
    }
    return options;
  }
}

import { HttpErrorResponse } from '@angular/common/http';
import { CommonModule } from '@angular/common';
import {
  Component,
  inject,
  OnDestroy,
  OnInit,
  signal,
} from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import {
  catchError,
  finalize,
  forkJoin,
  map,
  of,
  Subscription,
  switchMap,
} from 'rxjs';

import { AuthApiService, CurrentUser } from '../auth-api.service';
import { ProjectMember, ProjectMembersApiService } from '../project-members-api.service';
import {
  ProjectApiService,
  TASK_STATUS_OPTIONS,
  TaskDetail,
  TASK_PRIORITY_OPTIONS,
} from '../project-api.service';

type ValidTaskStatus = (typeof TASK_STATUS_OPTIONS)[number];

function normalizeTaskStatus(raw: string): ValidTaskStatus {
  return (TASK_STATUS_OPTIONS as readonly string[]).includes(raw as ValidTaskStatus)
    ? (raw as ValidTaskStatus)
    : 'Pending';
}

function normalizeTaskPriority(raw: string): string {
  return (TASK_PRIORITY_OPTIONS as readonly string[]).includes(raw)
    ? raw
    : 'Standard';
}

type PageResult =
  | { kind: 'invalid' }
  | {
      kind: 'ok';
      task: TaskDetail;
      currentUser: CurrentUser;
      assigneeOptions: Array<{ id: string; label: string }>;
    }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-task-edit',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './task-edit.html',
  styleUrls: [
    './task-edit.scss',
    '../feature-detail/feature-detail.scss',
    '../feature-create/feature-create.scss',
    '../task-create/task-create.scss',
  ],
})
export class TaskEdit implements OnInit, OnDestroy {
  private readonly fb = inject(FormBuilder);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly projectApi = inject(ProjectApiService);
  private readonly authApi = inject(AuthApiService);
  private readonly membersApi = inject(ProjectMembersApiService);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly taskMeta = signal<TaskDetail | null>(null);
  protected readonly currentUser = signal<CurrentUser | null>(null);
  protected readonly pageError = signal<string | null>(null);
  protected readonly assigneeOptions = signal<Array<{ id: string; label: string }>>([]);

  protected readonly statusOptions = [...TASK_STATUS_OPTIONS];
  protected readonly priorityOptions = [...TASK_PRIORITY_OPTIONS];

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly saveNotice = signal(false);

  protected readonly form = this.fb.nonNullable.group({
    title: ['', [Validators.required, Validators.maxLength(512)]],
    description: [''],
    status: this.fb.nonNullable.control<string>('Pending', Validators.required),
    priority: this.fb.nonNullable.control<string>('Standard', Validators.required),
    dueDate: this.fb.nonNullable.control<string>(''),
    dueTime: this.fb.nonNullable.control<string>(''),
    assigneeUserId: this.fb.nonNullable.control<string>(''),
  });

  ngOnInit(): void {
    this.sub = this.route.paramMap
      .pipe(
        switchMap((params) => {
          const projectId = params.get('projectId') ?? '';
          const featureId = params.get('featureId') ?? '';
          const taskId = params.get('taskId') ?? '';
          if (
            projectId.length === 0 ||
            featureId.length === 0 ||
            taskId.length === 0
          ) {
            return of<PageResult>({ kind: 'invalid' });
          }
          this.loadState.set('loading');
          this.pageError.set(null);
          this.serverError.set(null);
          this.saveNotice.set(false);
          return this.authApi.me().pipe(
            switchMap((currentUser) =>
              forkJoin({
                task: this.projectApi.getTask(projectId, featureId, taskId),
                members:
                  currentUser.accountType === 'company'
                    ? this.membersApi.listMembers(projectId)
                    : of([] as ProjectMember[]),
              }).pipe(
                map((data): PageResult => ({
                  kind: 'ok',
                  task: data.task,
                  currentUser,
                  assigneeOptions: this.buildAssigneeOptions(currentUser, data.members, data.task),
                })),
              ),
            ),
            catchError((err: unknown) =>
              of<PageResult>({
                kind: 'error',
                message:
                  err instanceof HttpErrorResponse && err.status === 401
                    ? 'You need to sign in again to edit this task.'
                    : ProjectApiService.taskDetailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.pageError.set('Missing project, feature, or task identifier.');
          this.taskMeta.set(null);
          this.currentUser.set(null);
          this.assigneeOptions.set([]);
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.taskMeta.set(null);
          this.currentUser.set(null);
          this.assigneeOptions.set([]);
          this.loadState.set('error');
          return;
        }
        this.taskMeta.set(res.task);
        this.currentUser.set(res.currentUser);
        this.assigneeOptions.set(res.assigneeOptions);
        this.pageError.set(null);
        this.form.patchValue({
          title: res.task.title,
          description: res.task.description ?? '',
          status: normalizeTaskStatus(res.task.status),
          priority: normalizeTaskPriority(res.task.priority),
          dueDate: res.task.dueDate ?? '',
          dueTime: this.normalizeDueTimeForInput(res.task.dueTime),
          assigneeUserId: res.task.assigneeUserId ?? '',
        });
        this.form.markAsPristine();
        this.saveNotice.set(false);
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  protected submit(): void {
    const t = this.taskMeta();
    if (!t) return;

    this.serverError.set(null);
    this.saveNotice.set(false);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting()) {
      return;
    }

    const cu = this.currentUser();
    if (!cu) {
      this.serverError.set('Could not resolve the current account. Please reload and sign in.');
      return;
    }

    const raw = this.form.getRawValue();
    const unassigned = raw.assigneeUserId === '';
    const dueDateTrimmed = raw.dueDate.trim();
    const dueTimeTrimmed = raw.dueTime.trim();
    if (dueDateTrimmed.length === 0 && dueTimeTrimmed.length > 0) {
      this.serverError.set('Due time cannot be set without a due date.');
      return;
    }
    this.submitting.set(true);

    this.projectApi
      .updateTask(t.projectId, t.featureId, t.id, {
        title: raw.title.trim(),
        description: raw.description,
        status: normalizeTaskStatus(raw.status),
        priority: raw.priority,
        dueDate: dueDateTrimmed.length > 0 ? dueDateTrimmed : undefined,
        dueTime: dueDateTrimmed.length > 0 && dueTimeTrimmed.length > 0 ? dueTimeTrimmed : undefined,
        clearDueDate: dueDateTrimmed.length === 0,
        unassigned,
        assigneeUserId: !unassigned ? raw.assigneeUserId : undefined,
      })
      .pipe(finalize(() => this.submitting.set(false)))
      .subscribe({
        next: (updated) => {
          this.taskMeta.set(updated);
          this.form.patchValue({
            title: updated.title,
            description: updated.description ?? '',
            status: normalizeTaskStatus(updated.status),
            priority: normalizeTaskPriority(updated.priority),
            dueDate: updated.dueDate ?? '',
            dueTime: this.normalizeDueTimeForInput(updated.dueTime),
            assigneeUserId: updated.assigneeUserId ?? '',
          });
          this.form.markAsPristine();
          this.saveNotice.set(true);
        },
        error: (err: unknown) => {
          this.serverError.set(ProjectApiService.updateTaskErrorMessage(err));
        },
      });
  }

  protected cancel(): void {
    const t = this.taskMeta();
    if (t) {
      void this.router.navigateByUrl(
        `/app/projects/${encodeURIComponent(t.projectId)}/features/${encodeURIComponent(t.featureId)}/tasks/${encodeURIComponent(t.id)}`,
      );
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

  private normalizeDueTimeForInput(value: string | null): string {
    if (!value) {
      return '';
    }
    return value.length >= 5 ? value.slice(0, 5) : value;
  }

  private buildAssigneeOptions(
    currentUser: CurrentUser,
    members: ProjectMember[],
    task: TaskDetail,
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

    if (
      task.assigneeUserId !== null &&
      !options.some((option) => option.id === task.assigneeUserId)
    ) {
      options.push({
        id: task.assigneeUserId,
        label: task.assigneeFullname?.trim() || 'Current assignee',
      });
    }
    return options;
  }

  protected shortTaskId(id: string): string {
    return id.replace(/-/g, '').slice(0, 8).toUpperCase();
  }
}

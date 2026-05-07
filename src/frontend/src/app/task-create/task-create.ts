import { HttpErrorResponse } from '@angular/common/http';
import { CommonModule } from '@angular/common';
import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { catchError, finalize, forkJoin, map, of, Subscription, switchMap } from 'rxjs';

import { AuthApiService, CurrentUser } from '../auth-api.service';
import {
  CreatedFeature,
  ProjectApiService,
  ProjectDetail as ProjectDetailModel,
  TASK_PRIORITY_OPTIONS,
} from '../project-api.service';

type PageLoadResult =
  | { kind: 'invalid' }
  | { kind: 'ok'; project: ProjectDetailModel; feature: CreatedFeature; currentUser: CurrentUser }
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
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly project = signal<ProjectDetailModel | null>(null);
  protected readonly feature = signal<CreatedFeature | null>(null);
  protected readonly currentUser = signal<CurrentUser | null>(null);
  protected readonly pageError = signal<string | null>(null);

  protected readonly priorityOptions = [...TASK_PRIORITY_OPTIONS];

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly showSuccess = signal(false);

  protected readonly form = this.fb.nonNullable.group({
    title: ['', [Validators.required, Validators.maxLength(512)]],
    description: [''],
    priority: this.fb.nonNullable.control<string>('Standard', Validators.required),
    assigneeMode: this.fb.nonNullable.control<'self' | 'none'>('self'),
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
          return forkJoin({
            project: this.projectApi.getProject(projectId),
            feature: this.projectApi.getFeature(projectId, featureId),
            currentUser: this.authApi.me(),
          }).pipe(
            map(
              (data): PageLoadResult => ({
                kind: 'ok',
                project: data.project,
                feature: data.feature,
                currentUser: data.currentUser,
              }),
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
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.project.set(null);
          this.feature.set(null);
          this.currentUser.set(null);
          this.loadState.set('error');
          return;
        }
        this.project.set(res.project);
        this.feature.set(res.feature);
        this.currentUser.set(res.currentUser);
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

    const { title, description, priority, assigneeMode } = this.form.getRawValue();
    const unassigned = assigneeMode === 'none';
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
        unassigned,
        assigneeUserId: !unassigned && cu ? cu.id : undefined,
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

  protected assigneeMeLabel(): string {
    const u = this.currentUser();
    return u?.fullname?.trim() ? u.fullname.trim() : 'Me';
  }
}

import { HttpErrorResponse } from '@angular/common/http';
import { CommonModule } from '@angular/common';
import {
  Component,
  computed,
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
import {
  ProjectApiService,
  TaskDetail,
  TASK_PRIORITY_OPTIONS,
} from '../project-api.service';

const VALID_TASK_STATUSES = ['Pending', 'In Progress', 'Done'] as const;
type ValidTaskStatus = (typeof VALID_TASK_STATUSES)[number];

function normalizeTaskStatus(raw: string): ValidTaskStatus {
  return VALID_TASK_STATUSES.includes(raw as ValidTaskStatus)
    ? (raw as ValidTaskStatus)
    : 'Pending';
}

function normalizeTaskPriority(raw: string): string {
  return (TASK_PRIORITY_OPTIONS as readonly string[]).includes(raw)
    ? raw
    : 'Standard';
}

function assigneeModeForTask(task: TaskDetail, currentUserId: string): 'self' | 'none' {
  const aid = task.assigneeUserId;
  if (aid === null || aid === '') {
    return 'none';
  }
  return aid === currentUserId ? 'self' : 'none';
}

type PageResult =
  | { kind: 'invalid' }
  | { kind: 'ok'; task: TaskDetail; currentUser: CurrentUser }
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
  private sub: Subscription | null = null;

  /** Local-only status change before Save; cleared on load and after successful PATCH. */
  private readonly statusLocalOverride = signal<string | null>(null);

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly taskMeta = signal<TaskDetail | null>(null);
  protected readonly currentUser = signal<CurrentUser | null>(null);
  protected readonly pageError = signal<string | null>(null);

  protected readonly priorityOptions = [...TASK_PRIORITY_OPTIONS];

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly saveNotice = signal(false);

  protected readonly effectiveStatus = computed(() => {
    const m = this.taskMeta();
    if (!m) {
      return 'Pending';
    }
    const o = this.statusLocalOverride();
    return normalizeTaskStatus(o ?? m.status);
  });

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
          this.statusLocalOverride.set(null);
          return forkJoin({
            task: this.projectApi.getTask(projectId, featureId, taskId),
            currentUser: this.authApi.me(),
          }).pipe(
            map((data): PageResult => ({
              kind: 'ok',
              task: data.task,
              currentUser: data.currentUser,
            })),
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
          this.statusLocalOverride.set(null);
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.taskMeta.set(null);
          this.currentUser.set(null);
          this.statusLocalOverride.set(null);
          this.loadState.set('error');
          return;
        }
        this.taskMeta.set(res.task);
        this.currentUser.set(res.currentUser);
        this.pageError.set(null);
        this.form.patchValue({
          title: res.task.title,
          description: res.task.description ?? '',
          priority: normalizeTaskPriority(res.task.priority),
          assigneeMode: assigneeModeForTask(res.task, res.currentUser.id),
        });
        this.form.markAsPristine();
        this.saveNotice.set(false);
        this.statusLocalOverride.set(null);
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  protected startProgress(): void {
    if (this.effectiveStatus() !== 'Pending') {
      return;
    }
    this.statusLocalOverride.set('In Progress');
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
    const unassigned = raw.assigneeMode === 'none';
    this.submitting.set(true);

    this.projectApi
      .updateTask(t.projectId, t.featureId, t.id, {
        title: raw.title.trim(),
        description: raw.description,
        status: normalizeTaskStatus(this.effectiveStatus()),
        priority: raw.priority,
        unassigned,
        assigneeUserId: !unassigned ? cu.id : undefined,
      })
      .pipe(finalize(() => this.submitting.set(false)))
      .subscribe({
        next: (updated) => {
          const u = this.currentUser();
          this.statusLocalOverride.set(null);
          this.taskMeta.set(updated);
          this.form.patchValue({
            title: updated.title,
            description: updated.description ?? '',
            priority: normalizeTaskPriority(updated.priority),
            assigneeMode:
              u === null ? 'none' : assigneeModeForTask(updated, u.id),
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

  protected assigneeMeLabel(): string {
    const u = this.currentUser();
    return u?.fullname?.trim() ? u.fullname.trim() : 'Me';
  }

  protected shortTaskId(id: string): string {
    return id.replace(/-/g, '').slice(0, 8).toUpperCase();
  }
}

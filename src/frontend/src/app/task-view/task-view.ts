import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { catchError, forkJoin, map, of, Subscription, switchMap } from 'rxjs';
import { AuthApiService, CurrentUser } from '../auth-api.service';

import {
  TASK_STATUS_OPTIONS,
  TASK_PRIORITY_OPTIONS,
  ProjectApiService,
  TaskDetail,
} from '../project-api.service';

type ValidTaskStatus = (typeof TASK_STATUS_OPTIONS)[number];

const VALID_TASK_PRIORITIES = TASK_PRIORITY_OPTIONS;

function normalizeTaskStatus(raw: string): string {
  return (TASK_STATUS_OPTIONS as readonly string[]).includes(raw as ValidTaskStatus) ? raw : 'Pending';
}

function normalizeTaskPriority(raw: string): string {
  return (VALID_TASK_PRIORITIES as readonly string[]).includes(raw) ? raw : 'Standard';
}

type PageResult =
  | { kind: 'invalid' }
  | { kind: 'ok'; detail: TaskDetail; currentUser: CurrentUser }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-task-view',
  imports: [RouterLink],
  templateUrl: './task-view.html',
  styleUrls: ['./task-view.scss', '../project-detail/project-detail.scss'],
})
export class TaskView implements OnInit, OnDestroy {
  private readonly route = inject(ActivatedRoute);
  private readonly projectApi = inject(ProjectApiService);
  private readonly authApi = inject(AuthApiService);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly taskDetail = signal<TaskDetail | null>(null);
  protected readonly currentUser = signal<CurrentUser | null>(null);
  protected readonly pageError = signal<string | null>(null);
  protected readonly descriptionExpanded = signal(false);

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
          this.descriptionExpanded.set(false);
          return forkJoin({
            detail: this.projectApi.getTask(projectId, featureId, taskId),
            currentUser: this.authApi.me(),
          }).pipe(
            map((data): PageResult => ({ kind: 'ok', detail: data.detail, currentUser: data.currentUser })),
            catchError((err: unknown) =>
              of<PageResult>({
                kind: 'error',
                message: ProjectApiService.taskDetailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.pageError.set('Missing project, feature, or task identifier.');
          this.taskDetail.set(null);
          this.currentUser.set(null);
          this.descriptionExpanded.set(false);
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.taskDetail.set(null);
          this.currentUser.set(null);
          this.descriptionExpanded.set(false);
          this.loadState.set('error');
          return;
        }
        this.taskDetail.set(res.detail);
        this.currentUser.set(res.currentUser);
        this.pageError.set(null);
        this.descriptionExpanded.set(false);
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  protected shortTaskId(id: string): string {
    return id.replace(/-/g, '').slice(0, 8).toUpperCase();
  }

  protected statusBadgeClass(status: string): string {
    switch (normalizeTaskStatus(status)) {
      case 'In Progress':
        return 'pd-status pd-status--progress';
      case 'Blocked':
        return 'pd-status pd-status--blocked';
      case 'In Review':
        return 'pd-status pd-status--review';
      case 'Done':
        return 'pd-status pd-status--done';
      case 'Cancelled':
        return 'pd-status pd-status--cancelled';
      default:
        return 'pd-status pd-status--pending';
    }
  }

  protected statusForDisplay(status: string): string {
    return normalizeTaskStatus(status).toUpperCase();
  }

  protected toggleDescriptionExpanded(): void {
    this.descriptionExpanded.update((v) => !v);
  }

  protected taskDescriptionText(description: string | null | undefined): string {
    const t = description?.trim();
    return t && t.length > 0 ? t : '';
  }

  protected hasExpandableDescription(description: string | null | undefined): boolean {
    return this.taskDescriptionText(description).length > 0;
  }

  protected formatDate(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) {
      return iso;
    }
    return d.toLocaleString(undefined, {
      dateStyle: 'medium',
      timeStyle: 'short',
    });
  }

  protected dueLabel(task: TaskDetail): string {
    if (!task.dueDate) {
      return 'No due date';
    }
    const value = task.dueTime ? `${task.dueDate}T${task.dueTime}` : `${task.dueDate}T00:00:00`;
    const d = new Date(value);
    if (Number.isNaN(d.getTime())) {
      return task.dueTime ? `${task.dueDate} ${task.dueTime}` : task.dueDate;
    }
    if (task.dueTime) {
      return d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
    }
    return d.toLocaleDateString(undefined, { dateStyle: 'medium' });
  }

  protected creatorPhotoSrc(task: TaskDetail): string {
    return task.creatorAvatarUrl?.trim() ?? '';
  }

  protected isAiOrigin(createdBy: string): boolean {
    return createdBy === 'AI';
  }

  protected creatorDisplayName(task: TaskDetail): string {
    const n = task.creatorFullname?.trim();
    return n && n.length > 0 ? n : 'Recorded user';
  }

  protected useCreatorPhoto(task: TaskDetail): boolean {
    const u = task.creatorAvatarUrl?.trim();
    return !!u && /^https?:\/\//i.test(u);
  }

  protected taskPriorityLabel(priority: string): string {
    return normalizeTaskPriority(priority);
  }

  protected taskPriorityIcon(priority: string): string {
    switch (normalizeTaskPriority(priority)) {
      case 'Elevated':
        return 'priority_high';
      case 'Critical':
        return 'error';
      default:
        return 'flag';
    }
  }

  protected taskPriorityIconClass(priority: string): string {
    const base = 'material-symbols-outlined pd-priority__icon';
    switch (normalizeTaskPriority(priority)) {
      case 'Elevated':
        return `${base} pd-priority__icon--high`;
      case 'Critical':
        return `${base} pd-priority__icon--high`;
      default:
        return `${base} pd-priority__icon--std`;
    }
  }

  protected canManageTasks(): boolean {
    const user = this.currentUser();
    if (!user) {
      return false;
    }
    return user.accountType !== 'company' || user.role !== 'viewer';
  }

  protected assigneeDisplayName(task: TaskDetail): string {
    const name = task.assigneeFullname?.trim();
    if (name && name.length > 0) {
      return name;
    }
    return task.assigneeUserId ? 'Assigned' : 'Unassigned';
  }

  protected useAssigneePhoto(task: TaskDetail): boolean {
    const url = task.assigneeAvatarUrl?.trim();
    return !!url && /^https?:\/\//i.test(url);
  }

  protected assigneePhotoSrc(task: TaskDetail): string {
    return task.assigneeAvatarUrl?.trim() ?? '';
  }
}

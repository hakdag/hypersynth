import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { catchError, forkJoin, map, of, Subscription, switchMap } from 'rxjs';
import { AuthApiService, CurrentUser } from '../auth-api.service';
import { CommentsApiService, TaskComment } from '../comments-api.service';

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
  imports: [RouterLink, FormsModule],
  templateUrl: './task-view.html',
  styleUrls: ['./task-view.scss', '../project-detail/project-detail.scss'],
})
export class TaskView implements OnInit, OnDestroy {
  private readonly route = inject(ActivatedRoute);
  private readonly projectApi = inject(ProjectApiService);
  private readonly authApi = inject(AuthApiService);
  private readonly commentsApi = inject(CommentsApiService);
  private sub: Subscription | null = null;
  private routeIds: { projectId: string; featureId: string; taskId: string } | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly taskDetail = signal<TaskDetail | null>(null);
  protected readonly currentUser = signal<CurrentUser | null>(null);
  protected readonly pageError = signal<string | null>(null);
  protected readonly descriptionExpanded = signal(false);
  protected readonly comments = signal<TaskComment[]>([]);
  protected readonly commentsLoadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly commentsError = signal<string | null>(null);
  protected readonly commentActionError = signal<string | null>(null);
  protected readonly creatingComment = signal(false);
  protected readonly deletingCommentId = signal<string | null>(null);
  protected readonly savingCommentId = signal<string | null>(null);
  protected readonly draftComment = signal('');
  protected readonly editingCommentId = signal<string | null>(null);
  protected readonly editCommentContent = signal('');

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
          this.comments.set([]);
          this.commentsError.set(null);
          this.commentsLoadState.set('loading');
          this.commentActionError.set(null);
          this.descriptionExpanded.set(false);
          this.loadState.set('error');
          return;
        }
        const projectId = this.route.snapshot.paramMap.get('projectId') ?? '';
        const featureId = this.route.snapshot.paramMap.get('featureId') ?? '';
        const taskId = this.route.snapshot.paramMap.get('taskId') ?? '';
        this.routeIds = { projectId, featureId, taskId };
        this.taskDetail.set(res.detail);
        this.currentUser.set(res.currentUser);
        this.pageError.set(null);
        this.descriptionExpanded.set(false);
        this.comments.set([]);
        this.commentsLoadState.set('loading');
        this.commentsError.set(null);
        this.commentActionError.set(null);
        this.draftComment.set('');
        this.editingCommentId.set(null);
        this.editCommentContent.set('');
        this.loadState.set('ok');
        this.loadComments();
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

  protected commentAuthorDisplayName(comment: TaskComment): string {
    const name = comment.authorFullname?.trim();
    return name && name.length > 0 ? name : 'Recorded user';
  }

  protected useCommentAuthorPhoto(comment: TaskComment): boolean {
    const url = comment.authorAvatarUrl?.trim();
    return !!url && /^https?:\/\//i.test(url);
  }

  protected commentAuthorPhotoSrc(comment: TaskComment): string {
    return comment.authorAvatarUrl?.trim() ?? '';
  }

  protected canEditComment(comment: TaskComment): boolean {
    const user = this.currentUser();
    if (!user) {
      return false;
    }
    return comment.userId === user.id || user.role === 'company_admin';
  }

  protected isCommentEdited(comment: TaskComment): boolean {
    return comment.updatedAt !== comment.createdAt;
  }

  protected beginEditComment(comment: TaskComment): void {
    if (!this.canEditComment(comment)) {
      return;
    }
    this.commentActionError.set(null);
    this.editingCommentId.set(comment.id);
    this.editCommentContent.set(comment.content);
  }

  protected cancelEditComment(): void {
    this.editingCommentId.set(null);
    this.editCommentContent.set('');
  }

  protected saveCommentEdit(comment: TaskComment): void {
    const ids = this.routeIds;
    if (!ids || !this.canEditComment(comment)) {
      return;
    }
    const content = this.editCommentContent().trim();
    if (!content) {
      this.commentActionError.set('Comment content is required.');
      return;
    }
    this.commentActionError.set(null);
    this.savingCommentId.set(comment.id);
    this.commentsApi
      .updateComment(ids.projectId, ids.featureId, ids.taskId, comment.id, { content })
      .subscribe({
        next: (updated) => {
          this.comments.update((rows) => rows.map((row) => (row.id === updated.id ? updated : row)));
          this.savingCommentId.set(null);
          this.editingCommentId.set(null);
          this.editCommentContent.set('');
        },
        error: (err: unknown) => {
          this.commentActionError.set(CommentsApiService.errorMessage(err));
          this.savingCommentId.set(null);
        },
      });
  }

  protected removeComment(comment: TaskComment): void {
    const ids = this.routeIds;
    if (!ids || !this.canEditComment(comment)) {
      return;
    }
    this.commentActionError.set(null);
    this.deletingCommentId.set(comment.id);
    this.commentsApi.deleteComment(ids.projectId, ids.featureId, ids.taskId, comment.id).subscribe({
      next: () => {
        this.comments.update((rows) => rows.filter((row) => row.id !== comment.id));
        this.deletingCommentId.set(null);
        if (this.editingCommentId() === comment.id) {
          this.cancelEditComment();
        }
      },
      error: (err: unknown) => {
        this.commentActionError.set(CommentsApiService.errorMessage(err));
        this.deletingCommentId.set(null);
      },
    });
  }

  protected submitComment(): void {
    const ids = this.routeIds;
    if (!ids) {
      return;
    }
    const content = this.draftComment().trim();
    if (!content) {
      this.commentActionError.set('Comment content is required.');
      return;
    }
    this.commentActionError.set(null);
    this.creatingComment.set(true);
    this.commentsApi.createComment(ids.projectId, ids.featureId, ids.taskId, { content }).subscribe({
      next: (comment) => {
        this.comments.update((rows) => [...rows, comment]);
        this.draftComment.set('');
        this.creatingComment.set(false);
      },
      error: (err: unknown) => {
        this.commentActionError.set(CommentsApiService.errorMessage(err));
        this.creatingComment.set(false);
      },
    });
  }

  protected reloadComments(): void {
    this.loadComments();
  }

  private loadComments(): void {
    const ids = this.routeIds;
    if (!ids) {
      this.comments.set([]);
      this.commentsLoadState.set('error');
      this.commentsError.set('Missing project, feature, or task identifier.');
      return;
    }
    this.commentsLoadState.set('loading');
    this.commentsError.set(null);
    this.commentsApi.listComments(ids.projectId, ids.featureId, ids.taskId).subscribe({
      next: (rows) => {
        this.comments.set(rows);
        this.commentsLoadState.set('ok');
      },
      error: (err: unknown) => {
        this.comments.set([]);
        this.commentsError.set(CommentsApiService.errorMessage(err));
        this.commentsLoadState.set('error');
      },
    });
  }
}

import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import {
  catchError,
  forkJoin,
  map,
  of,
  Subscription,
  switchMap,
} from 'rxjs';

import {
  CreatedFeature,
  CreatedTask,
  ProjectApiService,
  ProjectDetail as ProjectDetailModel,
  TASK_PRIORITY_OPTIONS,
} from '../project-api.service';
import { AuthApiService, CurrentUser } from '../auth-api.service';
import { TaskAiGenerateDialog } from '../task-ai-generate-dialog/task-ai-generate-dialog';

const VALID_FEATURE_STATUSES = ['Pending', 'In Progress', 'Done'] as const;
type ValidFeatureStatus = (typeof VALID_FEATURE_STATUSES)[number];

function normalizeStatus(raw: string): string {
  return VALID_FEATURE_STATUSES.includes(raw as ValidFeatureStatus) ? raw : 'Pending';
}

const VALID_TASK_STATUSES = ['Pending', 'In Progress', 'Done'] as const;
type ValidTaskStatus = (typeof VALID_TASK_STATUSES)[number];

function normalizeTaskStatus(raw: string): string {
  return VALID_TASK_STATUSES.includes(raw as ValidTaskStatus) ? raw : 'Pending';
}

type ValidTaskPriority = (typeof TASK_PRIORITY_OPTIONS)[number];

function normalizeTaskPriority(raw: string): string {
  return (TASK_PRIORITY_OPTIONS as readonly string[]).includes(raw as ValidTaskPriority)
    ? raw
    : 'Standard';
}

type PageResult =
  | { kind: 'invalid' }
  | {
      kind: 'ok';
      project: ProjectDetailModel;
      feature: CreatedFeature;
      tasks: CreatedTask[];
      currentUser: CurrentUser;
    }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-feature-view',
  imports: [RouterLink, TaskAiGenerateDialog],
  templateUrl: './feature-view.html',
  styleUrls: ['./feature-view.scss', '../project-detail/project-detail.scss'],
})
export class FeatureView implements OnInit, OnDestroy {
  private readonly route = inject(ActivatedRoute);
  private readonly projectApi = inject(ProjectApiService);
  private readonly authApi = inject(AuthApiService);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly project = signal<ProjectDetailModel | null>(null);
  protected readonly featureMeta = signal<CreatedFeature | null>(null);
  protected readonly taskList = signal<CreatedTask[]>([]);
  protected readonly currentUser = signal<CurrentUser | null>(null);
  protected readonly pageError = signal<string | null>(null);
  protected readonly requirementsExpanded = signal(false);
  protected readonly requirementsCopyFlash = signal(false);
  protected readonly aiTaskDialogOpen = signal(false);

  private requirementsCopyTimer: ReturnType<typeof setTimeout> | null = null;

  ngOnInit(): void {
    this.sub = this.route.paramMap
      .pipe(
        switchMap((params) => {
          const projectId = params.get('projectId') ?? '';
          const featureId = params.get('featureId') ?? '';
          if (projectId.length === 0 || featureId.length === 0) {
            return of<PageResult>({ kind: 'invalid' });
          }
          this.loadState.set('loading');
          this.pageError.set(null);
          this.requirementsExpanded.set(false);
          this.clearRequirementsCopyFlash();
          return forkJoin({
            project: this.projectApi.getProject(projectId),
            feature: this.projectApi.getFeature(projectId, featureId),
            tasks: this.projectApi.listTasks(projectId, featureId),
            currentUser: this.authApi.me(),
          }).pipe(
            map(
              (data): PageResult => ({
                kind: 'ok',
                project: data.project,
                feature: data.feature,
                tasks: data.tasks,
                currentUser: data.currentUser,
              }),
            ),
            catchError((err: unknown) =>
              of<PageResult>({
                kind: 'error',
                message: ProjectApiService.featureDetailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.pageError.set('Missing project or feature identifier.');
          this.project.set(null);
          this.featureMeta.set(null);
          this.taskList.set([]);
          this.currentUser.set(null);
          this.requirementsExpanded.set(false);
          this.aiTaskDialogOpen.set(false);
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.project.set(null);
          this.featureMeta.set(null);
          this.taskList.set([]);
          this.currentUser.set(null);
          this.requirementsExpanded.set(false);
          this.aiTaskDialogOpen.set(false);
          this.loadState.set('error');
          return;
        }
        this.project.set(res.project);
        this.featureMeta.set(res.feature);
        this.taskList.set(res.tasks);
        this.currentUser.set(res.currentUser);
        this.pageError.set(null);
          this.requirementsExpanded.set(false);
          this.aiTaskDialogOpen.set(false);
          this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
    this.clearRequirementsCopyFlash();
  }

  private clearRequirementsCopyFlash(): void {
    if (this.requirementsCopyTimer !== null) {
      clearTimeout(this.requirementsCopyTimer);
      this.requirementsCopyTimer = null;
    }
    this.requirementsCopyFlash.set(false);
  }

  protected copyFeatureRequirements(requirements: string | null | undefined): void {
    const text = this.featureRequirementsText(requirements);
    if (text.length === 0) {
      return;
    }
    void navigator.clipboard.writeText(text).then(() => {
      if (this.requirementsCopyTimer !== null) {
        clearTimeout(this.requirementsCopyTimer);
        this.requirementsCopyTimer = null;
      }
      this.requirementsCopyFlash.set(true);
      this.requirementsCopyTimer = setTimeout(() => {
        this.requirementsCopyFlash.set(false);
        this.requirementsCopyTimer = null;
      }, 1600);
    });
  }

  protected shortFeatureId(id: string): string {
    return id.replace(/-/g, '').slice(0, 8).toUpperCase();
  }

  protected completionPercent(status: string): number {
    switch (normalizeStatus(status)) {
      case 'In Progress':
        return 50;
      case 'Done':
        return 100;
      default:
        return 0;
    }
  }

  protected priorityLabel(status: string): string {
    switch (normalizeStatus(status)) {
      case 'In Progress':
        return 'Elevated';
      case 'Done':
        return 'Complete';
      default:
        return 'Standard';
    }
  }

  protected priorityIcon(status: string): string {
    switch (normalizeStatus(status)) {
      case 'In Progress':
        return 'priority_high';
      case 'Done':
        return 'check_circle';
      default:
        return 'flag';
    }
  }

  protected priorityIconClass(status: string): string {
    const base = 'material-symbols-outlined pd-priority__icon';
    switch (normalizeStatus(status)) {
      case 'In Progress':
        return `${base} pd-priority__icon--high`;
      case 'Done':
        return `${base} pd-priority__icon--done`;
      default:
        return `${base} pd-priority__icon--std`;
    }
  }

  protected statusBadgeClass(status: string): string {
    switch (normalizeStatus(status)) {
      case 'In Progress':
        return 'pd-status pd-status--progress';
      case 'Done':
        return 'pd-status pd-status--done';
      default:
        return 'pd-status pd-status--pending';
    }
  }

  protected statusForDisplay(status: string): string {
    return normalizeStatus(status).toUpperCase();
  }

  protected toggleRequirementsExpanded(): void {
    this.requirementsExpanded.update((v) => !v);
  }

  protected featureRequirementsText(requirements: string | null | undefined): string {
    const t = requirements?.trim();
    return t && t.length > 0 ? t : '';
  }

  protected hasExpandableFeatureRequirements(requirements: string | null | undefined): boolean {
    return this.featureRequirementsText(requirements).length > 0;
  }

  protected taskStatusBadgeClass(status: string): string {
    switch (normalizeTaskStatus(status)) {
      case 'In Progress':
        return 'fv-task-badge fv-task-badge--progress';
      case 'Done':
        return 'fv-task-badge fv-task-badge--done';
      default:
        return 'fv-task-badge fv-task-badge--pending';
    }
  }

  protected taskStatusLabel(status: string): string {
    return normalizeTaskStatus(status);
  }

  protected taskPriorityLabel(priority: string): string {
    return normalizeTaskPriority(priority);
  }

  protected taskPriorityIcon(priority: string): string {
    switch (normalizeTaskPriority(priority)) {
      case 'Elevated':
        return 'priority_high';
      case 'Critical':
        return 'warning';
      default:
        return 'flag';
    }
  }

  protected taskPriorityIconClass(priority: string): string {
    const base = 'material-symbols-outlined pd-priority__icon';
    switch (normalizeTaskPriority(priority)) {
      case 'Elevated':
      case 'Critical':
        return `${base} pd-priority__icon--high`;
      default:
        return `${base} pd-priority__icon--std`;
    }
  }

  protected taskCreatedByLabel(createdBy: string): string {
    if (createdBy === 'User') {
      return 'Manual';
    }
    if (createdBy === 'AI') {
      return 'AI generated';
    }
    return createdBy;
  }

  protected taskCreatedByPresentation(createdBy: string): 'manual' | 'ai' | 'other' {
    if (createdBy === 'User') {
      return 'manual';
    }
    if (createdBy === 'AI') {
      return 'ai';
    }
    return 'other';
  }

  protected taskTitleRowClass(status: string): string {
    return normalizeTaskStatus(status) === 'Done' ? 'fv-task-title fv-task-title--done' : 'fv-task-title';
  }

  protected canManageTasks(): boolean {
    const user = this.currentUser();
    if (!user) {
      return false;
    }
    return user.accountType !== 'company' || user.role !== 'viewer';
  }

  protected taskAssigneeLabel(task: CreatedTask): string {
    const name = task.assigneeFullname?.trim();
    if (name && name.length > 0) {
      return name;
    }
    return task.assigneeUserId ? 'Assigned' : 'Unassigned';
  }

  protected taskDueLabel(task: CreatedTask): string {
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

  protected taskDueClass(task: CreatedTask): string {
    return task.isOverdue ? 'fv-task-due fv-task-due--overdue' : 'fv-task-due';
  }

  protected openAiTaskDialog(): void {
    this.aiTaskDialogOpen.set(true);
  }

  protected closeAiTaskDialog(): void {
    this.aiTaskDialogOpen.set(false);
  }

  protected onAiTasksAccepted(): void {
    const p = this.project();
    const f = this.featureMeta();
    if (!p || !f) {
      this.aiTaskDialogOpen.set(false);
      return;
    }
    this.aiTaskDialogOpen.set(false);
    this.projectApi.listTasks(p.id, f.id).subscribe({
      next: (tasks) => this.taskList.set(tasks),
      error: (err: unknown) => {
        this.pageError.set(ProjectApiService.listTasksErrorMessage(err));
      },
    });
  }

  protected aiGenerateDisabledReason(
    proj: ProjectDetailModel,
    meta: CreatedFeature,
  ): string {
    if (!proj.hasAiApiKey) {
      return 'Configure an AI API key on the project before generating tasks.';
    }
    if (this.featureRequirementsText(meta.requirements).length === 0) {
      return 'Add feature requirements before generating tasks with AI.';
    }
    return '';
  }

  protected aiGenerateDisabled(proj: ProjectDetailModel, meta: CreatedFeature): boolean {
    return this.aiGenerateDisabledReason(proj, meta).length > 0;
  }
}

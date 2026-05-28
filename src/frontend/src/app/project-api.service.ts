import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface CreateProjectPayload {
  name: string;
  requirements?: string;
}

export interface UpdateProjectPayload {
  name: string;
  requirements: string;
  status: string;
}

export interface CreatedProject {
  id: string;
  ownerUserId: string | null;
  companyId: string | null;
  name: string;
  requirements: string | null;
  status: string;
  createdAt: string;
}

export interface ProjectDetail extends CreatedProject {
  hasAiApiKey: boolean;
  canManageAiSettings: boolean;
}

export type AiProviderId = 'anthropic' | 'openai';

export interface ProviderCatalogResponse {
  providers: AiProviderId[];
}

export interface ProjectAiSettings {
  projectId: string;
  provider: AiProviderId | null;
  allowedModels: string[];
  monthlyTokenLimit: number | null;
  usageTrackingEnabled: boolean;
  hasApiKey: boolean;
  apiKeyHint: string | null;
}

export interface UpdateProjectAiSettingsPayload {
  provider: AiProviderId;
  allowedModels: string[];
  monthlyTokenLimit: number | null;
  usageTrackingEnabled: boolean;
  apiKey?: string;
  clearApiKey: boolean;
}

export interface ListProviderModelsResponse {
  provider: AiProviderId;
  models: string[];
}

export interface EnhanceProjectRequirementsResponse {
  enhancedRequirements: string;
}

export interface GeneratedTaskCandidate {
  title: string;
  description: string;
}

export interface TaskGenerationTurn {
  proposedTasks: GeneratedTaskCandidate[];
  feedback: string;
}

export interface GenerateTasksResponse {
  tasks: GeneratedTaskCandidate[];
}

export interface CreatedFeature {
  id: string;
  projectId: string;
  title: string;
  requirements: string | null;
  status: string;
  createdAt: string;
}

export interface ProjectDocument {
  id: string;
  projectId: string;
  filePath: string;
  metadata: Record<string, unknown>;
  createdAt: string;
}

export interface UpdateFeaturePayload {
  title: string;
  requirements: string;
  status: string;
}

export interface UpdateTaskPayload {
  title: string;
  description: string;
  status: string;
  priority: string;
  dueDate?: string;
  dueTime?: string;
  clearDueDate?: boolean;
  unassigned: boolean;
  assigneeUserId?: string;
}

export const TASK_STATUS_OPTIONS = [
  'Pending',
  'In Progress',
  'Blocked',
  'In Review',
  'Done',
  'Cancelled',
] as const;
export type TaskStatus = (typeof TASK_STATUS_OPTIONS)[number];
export const TERMINAL_TASK_STATUSES = ['Done', 'Cancelled'] as const;

export const TASK_PRIORITY_OPTIONS = ['Standard', 'Elevated', 'Critical'] as const;
export type TaskPriority = (typeof TASK_PRIORITY_OPTIONS)[number];

export interface CreatedTask {
  id: string;
  featureId: string;
  title: string;
  description: string | null;
  status: string;
  createdBy: string;
  createdAt: string;
  priority: string;
  dueDate: string | null;
  dueTime: string | null;
  isOverdue: boolean;
  assigneeUserId: string | null;
  assigneeFullname: string | null;
  assigneeAvatarUrl: string | null;
  creatorFullname: string | null;
  creatorAvatarUrl: string | null;
}

export interface TaskDetail extends CreatedTask {
  featureTitle: string;
  projectId: string;
  projectName: string;
}

@Injectable({
  providedIn: 'root',
})
export class ProjectApiService {
  private readonly http = inject(HttpClient);

  listProjects(): Observable<CreatedProject[]> {
    const url = `${environment.apiBaseUrl}/api/v1/projects`;
    return this.http.get<CreatedProject[]>(url);
  }

  getProject(id: string): Observable<ProjectDetail> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(id)}`;
    return this.http.get<ProjectDetail>(url);
  }

  createProject(payload: CreateProjectPayload): Observable<CreatedProject> {
    const body: Record<string, unknown> = { name: payload.name };
    if (payload.requirements !== undefined && payload.requirements.length > 0) {
      body['requirements'] = payload.requirements;
    }
    const url = `${environment.apiBaseUrl}/api/v1/projects`;
    return this.http.post<CreatedProject>(url, body);
  }

  updateProject(id: string, payload: UpdateProjectPayload): Observable<CreatedProject> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(id)}`;
    const body = {
      name: payload.name.trim(),
      requirements: payload.requirements,
      status: payload.status,
    };
    return this.http.patch<CreatedProject>(url, body);
  }

  listAiProviders(): Observable<ProviderCatalogResponse> {
    const url = `${environment.apiBaseUrl}/api/v1/ai/providers`;
    return this.http.get<ProviderCatalogResponse>(url);
  }

  getAiSettings(projectId: string): Observable<ProjectAiSettings> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(projectId)}/ai-settings`;
    return this.http.get<ProjectAiSettings>(url);
  }

  updateAiSettings(
    projectId: string,
    payload: UpdateProjectAiSettingsPayload,
  ): Observable<ProjectAiSettings> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(projectId)}/ai-settings`;
    const body: Record<string, unknown> = {
      provider: payload.provider,
      allowedModels: payload.allowedModels,
      monthlyTokenLimit: payload.monthlyTokenLimit,
      usageTrackingEnabled: payload.usageTrackingEnabled,
      clearApiKey: payload.clearApiKey,
    };
    if (payload.apiKey !== undefined && payload.apiKey.trim().length > 0) {
      body['apiKey'] = payload.apiKey.trim();
    }
    return this.http.put<ProjectAiSettings>(url, body);
  }

  fetchProviderModels(
    projectId: string,
    provider: AiProviderId,
    apiKey: string,
  ): Observable<ListProviderModelsResponse> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(projectId)}/ai-settings/provider-models`;
    return this.http.post<ListProviderModelsResponse>(url, {
      provider,
      apiKey,
    });
  }

  enhanceProjectRequirements(
    projectId: string,
    documentIds: string[] = [],
  ): Observable<EnhanceProjectRequirementsResponse> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(projectId)}/ai/enhance-requirements`;
    return this.http.post<EnhanceProjectRequirementsResponse>(url, { documentIds });
  }

  enhanceFeatureRequirements(
    projectId: string,
    featureId: string,
    documentIds: string[] = [],
  ): Observable<EnhanceProjectRequirementsResponse> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/ai/enhance-requirements`;
    return this.http.post<EnhanceProjectRequirementsResponse>(url, { documentIds });
  }

  generateFeatureTasks(
    projectId: string,
    featureId: string,
    feedbackHistory: TaskGenerationTurn[],
    documentIds: string[] = [],
  ): Observable<GenerateTasksResponse> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/ai/generate-tasks`;
    return this.http.post<GenerateTasksResponse>(url, { feedbackHistory, documentIds });
  }

  acceptGeneratedTasks(
    projectId: string,
    featureId: string,
    tasks: GeneratedTaskCandidate[],
  ): Observable<CreatedTask[]> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/ai/accept-tasks`;
    return this.http.post<CreatedTask[]>(url, { tasks });
  }

  createFeature(
    projectId: string,
    payload: { title: string; requirements?: string },
  ): Observable<CreatedFeature> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(projectId)}/features`;
    const body: Record<string, unknown> = { title: payload.title.trim() };
    if (payload.requirements !== undefined && payload.requirements.trim().length > 0) {
      body['requirements'] = payload.requirements.trim();
    }
    return this.http.post<CreatedFeature>(url, body);
  }

  listFeatures(projectId: string): Observable<CreatedFeature[]> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(projectId)}/features`;
    return this.http.get<CreatedFeature[]>(url);
  }

  listProjectDocuments(projectId: string): Observable<ProjectDocument[]> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(projectId)}/documents`;
    return this.http.get<ProjectDocument[]>(url);
  }

  uploadProjectDocuments(projectId: string, files: File[]): Observable<ProjectDocument[]> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${encodeURIComponent(projectId)}/documents`;
    const body = new FormData();
    for (const file of files) {
      body.append('files', file);
    }
    return this.http.post<ProjectDocument[]>(url, body);
  }

  downloadProjectDocument(projectId: string, documentId: string): Observable<Blob> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/documents/${b(documentId)}/download`;
    return this.http.get(url, { responseType: 'blob' });
  }

  viewProjectDocument(projectId: string, documentId: string): Observable<Blob> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/documents/${b(documentId)}/download`;
    return this.http.get(url, { responseType: 'blob' });
  }

  getFeature(projectId: string, featureId: string): Observable<CreatedFeature> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}`;
    return this.http.get<CreatedFeature>(url);
  }

  updateFeature(
    projectId: string,
    featureId: string,
    payload: UpdateFeaturePayload,
  ): Observable<CreatedFeature> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}`;
    const body: Record<string, unknown> = {
      title: payload.title.trim(),
      status: payload.status,
    };
    body['requirements'] =
      payload.requirements.trim().length > 0 ? payload.requirements.trim() : null;
    return this.http.patch<CreatedFeature>(url, body);
  }

  createTask(
    projectId: string,
    featureId: string,
    payload: {
      title: string;
      description?: string;
      priority: string;
      dueDate?: string;
      dueTime?: string;
      unassigned: boolean;
      assigneeUserId?: string;
    },
  ): Observable<CreatedTask> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/tasks`;
    const body: Record<string, unknown> = {
      title: payload.title.trim(),
      priority: payload.priority,
      unassigned: payload.unassigned,
    };
    if (payload.description !== undefined && payload.description.trim().length > 0) {
      body['description'] = payload.description.trim();
    }
    if (payload.dueDate !== undefined && payload.dueDate.trim().length > 0) {
      body['dueDate'] = payload.dueDate.trim();
      if (payload.dueTime !== undefined && payload.dueTime.trim().length > 0) {
        body['dueTime'] = payload.dueTime.trim();
      }
    }
    if (!payload.unassigned && payload.assigneeUserId !== undefined) {
      body['assigneeUserId'] = payload.assigneeUserId;
    }
    return this.http.post<CreatedTask>(url, body);
  }

  listTasks(projectId: string, featureId: string): Observable<CreatedTask[]> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/tasks`;
    return this.http.get<CreatedTask[]>(url);
  }

  getTask(projectId: string, featureId: string, taskId: string): Observable<TaskDetail> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/tasks/${b(taskId)}`;
    return this.http.get<TaskDetail>(url);
  }

  updateTask(
    projectId: string,
    featureId: string,
    taskId: string,
    payload: UpdateTaskPayload,
  ): Observable<TaskDetail> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/tasks/${b(taskId)}`;
    const body: Record<string, unknown> = {
      title: payload.title.trim(),
      status: payload.status,
      priority: payload.priority,
      clearDueDate: payload.clearDueDate === true,
      unassigned: payload.unassigned,
    };
    body['description'] =
      payload.description.trim().length > 0 ? payload.description.trim() : null;
    if (payload.clearDueDate !== true && payload.dueDate !== undefined && payload.dueDate.trim().length > 0) {
      body['dueDate'] = payload.dueDate.trim();
      if (payload.dueTime !== undefined && payload.dueTime.trim().length > 0) {
        body['dueTime'] = payload.dueTime.trim();
      }
    }
    if (!payload.unassigned && payload.assigneeUserId !== undefined) {
      body['assigneeUserId'] = payload.assigneeUserId;
    }
    return this.http.patch<TaskDetail>(url, body);
  }

  static listErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return `Could not load projects (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static detailErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Project not found or you do not have access.';
      }
      return `Could not load project (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return `Could not create project (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static listFeaturesErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Project not found or you do not have access.';
      }
      return `Could not load features (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  /** Message when loading documents for AI context picker (same rules as listing). */
  static loadDocumentsErrorMessage(err: unknown): string {
    return ProjectApiService.listDocumentsErrorMessage(err);
  }

  static listDocumentsErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Project not found or you do not have access.';
      }
      return `Could not load documents (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static uploadDocumentsErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Project not found or you do not have access.';
      }
      return `Could not upload documents (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static downloadDocumentErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      if (err.status === 404) {
        return 'Document not found or you do not have access.';
      }
      return `Could not download document (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static viewDocumentErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      if (err.status === 404) {
        return 'Document not found or you do not have access.';
      }
      return `Could not load document preview (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static featureDetailErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Feature not found or you do not have access.';
      }
      return `Could not load feature (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static createFeatureErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Project not found or you do not have access.';
      }
      return `Could not create feature (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static updateFeatureErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Feature not found or you do not have access.';
      }
      return `Could not save feature (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static createTaskErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Feature not found or you do not have access.';
      }
      return `Could not create task (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static listTasksErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Feature not found or you do not have access.';
      }
      return `Could not load tasks (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static taskDetailErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Task not found or you do not have access.';
      }
      return `Could not load task (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static updateTaskErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Task not found or you do not have access.';
      }
      return `Could not save task (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static updateErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Project not found or you do not have access.';
      }
      return `Could not save project (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static aiSettingsErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Project not found or you do not have access.';
      }
      return `Could not save AI settings (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static listProviderModelsErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return `Could not fetch provider models (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static enhanceErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Project not found or you do not have access.';
      }
      return `Could not enhance requirements (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static enhanceFeatureRequirementsErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Feature not found or you do not have access.';
      }
      return `Could not enhance feature requirements (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static generateTasksErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Feature not found or you do not have access.';
      }
      return `Could not generate tasks (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }

  static acceptGeneratedTasksErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      if (err.status === 404) {
        return 'Feature not found or you do not have access.';
      }
      return `Could not save generated tasks (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }
}

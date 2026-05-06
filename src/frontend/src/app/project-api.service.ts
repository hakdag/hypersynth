import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface CreateProjectPayload {
  name: string;
  requirements?: string;
  aiApiKey?: string;
}

export interface UpdateProjectPayload {
  name: string;
  requirements: string;
  status: string;
  clearAiApiKey: boolean;
  aiApiKey: string;
}

export interface CreatedProject {
  id: string;
  userId: string;
  name: string;
  requirements: string | null;
  status: string;
  createdAt: string;
}

export interface ProjectDetail extends CreatedProject {
  hasAiApiKey: boolean;
}

export interface CreatedFeature {
  id: string;
  projectId: string;
  title: string;
  requirements: string | null;
  status: string;
  createdAt: string;
}

export interface UpdateFeaturePayload {
  title: string;
  requirements: string;
  status: string;
}

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
    if (payload.aiApiKey !== undefined && payload.aiApiKey.length > 0) {
      body['aiApiKey'] = payload.aiApiKey;
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
      clearAiApiKey: payload.clearAiApiKey,
      aiApiKey: payload.aiApiKey.trim().length > 0 ? payload.aiApiKey.trim() : null,
    };
    return this.http.patch<CreatedProject>(url, body);
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
}

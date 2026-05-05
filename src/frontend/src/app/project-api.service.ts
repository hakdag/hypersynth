import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface CreateProjectPayload {
  name: string;
  requirements?: string;
  aiApiKey?: string;
}

export interface CreatedProject {
  id: string;
  userId: string;
  name: string;
  requirements: string | null;
  status: string;
  createdAt: string;
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
}

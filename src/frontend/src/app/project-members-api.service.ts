import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import type { CompanyRole } from './auth-api.service';
import { environment } from '../environments/environment';

export type ProjectMembershipRole = 'project_manager' | 'contributor' | 'viewer';

export interface ProjectMember {
  userId: string;
  fullname: string;
  email: string;
  companyRole: CompanyRole | null;
  projectRole: ProjectMembershipRole;
  createdAt: string;
}

export interface AddProjectMemberPayload {
  userId: string;
  projectRole: ProjectMembershipRole;
}

@Injectable({
  providedIn: 'root',
})
export class ProjectMembersApiService {
  private readonly http = inject(HttpClient);

  listMembers(projectId: string): Observable<ProjectMember[]> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${projectId}/members`;
    return this.http.get<ProjectMember[]>(url);
  }

  addMember(projectId: string, payload: AddProjectMemberPayload): Observable<ProjectMember> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${projectId}/members`;
    return this.http.post<ProjectMember>(url, payload);
  }

  removeMember(projectId: string, userId: string): Observable<void> {
    const url = `${environment.apiBaseUrl}/api/v1/projects/${projectId}/members/${userId}`;
    return this.http.delete<void>(url);
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return `Request failed (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }
}

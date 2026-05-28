import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface CommentMention {
  userId: string;
  username: string;
  fullname: string;
}

export interface TaskComment {
  id: string;
  taskId: string;
  userId: string;
  authorFullname: string;
  authorAvatarUrl: string | null;
  content: string;
  createdAt: string;
  updatedAt: string;
  mentions: CommentMention[];
}

@Injectable({
  providedIn: 'root',
})
export class CommentsApiService {
  private readonly http = inject(HttpClient);

  listComments(projectId: string, featureId: string, taskId: string): Observable<TaskComment[]> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/tasks/${b(taskId)}/comments`;
    return this.http.get<TaskComment[]>(url);
  }

  createComment(
    projectId: string,
    featureId: string,
    taskId: string,
    payload: { content: string },
  ): Observable<TaskComment> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/tasks/${b(taskId)}/comments`;
    return this.http.post<TaskComment>(url, { content: payload.content.trim() });
  }

  updateComment(
    projectId: string,
    featureId: string,
    taskId: string,
    commentId: string,
    payload: { content: string },
  ): Observable<TaskComment> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/tasks/${b(taskId)}/comments/${b(commentId)}`;
    return this.http.patch<TaskComment>(url, { content: payload.content.trim() });
  }

  deleteComment(
    projectId: string,
    featureId: string,
    taskId: string,
    commentId: string,
  ): Observable<void> {
    const b = encodeURIComponent;
    const url = `${environment.apiBaseUrl}/api/v1/projects/${b(projectId)}/features/${b(featureId)}/tasks/${b(taskId)}/comments/${b(commentId)}`;
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

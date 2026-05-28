import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface Label {
  id: string;
  name: string;
  color: string;
  createdAt: string;
}

export interface CreateLabelPayload {
  name: string;
  color: string;
}

export interface UpdateLabelPayload {
  name: string;
  color: string;
}

@Injectable({
  providedIn: 'root',
})
export class LabelsApiService {
  private readonly http = inject(HttpClient);

  listLabels(): Observable<Label[]> {
    const url = `${environment.apiBaseUrl}/api/v1/labels`;
    return this.http.get<Label[]>(url);
  }

  createLabel(payload: CreateLabelPayload): Observable<Label> {
    const url = `${environment.apiBaseUrl}/api/v1/labels`;
    return this.http.post<Label>(url, payload);
  }

  updateLabel(labelId: string, payload: UpdateLabelPayload): Observable<Label> {
    const url = `${environment.apiBaseUrl}/api/v1/labels/${encodeURIComponent(labelId)}`;
    return this.http.patch<Label>(url, payload);
  }

  deleteLabel(labelId: string): Observable<void> {
    const url = `${environment.apiBaseUrl}/api/v1/labels/${encodeURIComponent(labelId)}`;
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

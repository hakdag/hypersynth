import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface CurrentUser {
  id: string;
  fullname: string;
  email: string;
  avatarUrl: string | null;
}

@Injectable({
  providedIn: 'root',
})
export class AuthApiService {
  private readonly http = inject(HttpClient);

  login(payload: { email: string; password: string }): Observable<CurrentUser> {
    const url = `${environment.apiBaseUrl}/api/v1/login`;
    return this.http.post<CurrentUser>(url, payload);
  }

  logout(): Observable<void> {
    const url = `${environment.apiBaseUrl}/api/v1/logout`;
    return this.http.post<void>(url, {});
  }

  me(): Observable<CurrentUser> {
    const url = `${environment.apiBaseUrl}/api/v1/me`;
    return this.http.get<CurrentUser>(url);
  }

  static loginErrorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return 'Could not sign in. Please try again.';
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }
}

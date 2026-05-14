import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import type { CompanyRole } from './auth-api.service';
import { environment } from '../environments/environment';

export interface CompanyUser {
  id: string;
  fullname: string;
  email: string;
  role: CompanyRole | null;
}

@Injectable({
  providedIn: 'root',
})
export class CompanyUsersApiService {
  private readonly http = inject(HttpClient);

  listCompanyUsers(): Observable<CompanyUser[]> {
    const url = `${environment.apiBaseUrl}/api/v1/company/users`;
    return this.http.get<CompanyUser[]>(url);
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

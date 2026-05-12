import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export type RegisterAccountType = 'personal' | 'company';

export interface RegisterPayload {
  accountType: RegisterAccountType;
  fullname: string;
  email: string;
  password: string;
}

export interface RegisterSuccess {
  id: string;
  message: string;
}

export interface CompanyRegistrationPayload {
  name: string;
  companyEmail: string;
  country: string;
  timezone: string;
  fullName: string;
  email: string;
  username: string;
  password: string;
  passwordConfirmation: string;
}

export interface CompanyRegistrationSuccess {
  userId: string;
  companyId: string;
  message: string;
}

@Injectable({
  providedIn: 'root',
})
export class RegistrationApiService {
  private readonly http = inject(HttpClient);

  register(payload: RegisterPayload): Observable<RegisterSuccess> {
    const url = `${environment.apiBaseUrl}/api/v1/register`;
    return this.http.post<RegisterSuccess>(url, payload);
  }

  registerCompany(payload: CompanyRegistrationPayload): Observable<CompanyRegistrationSuccess> {
    const url = `${environment.apiBaseUrl}/api/v1/companies/register`;
    return this.http.post<CompanyRegistrationSuccess>(url, payload);
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return `Could not register (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }
}

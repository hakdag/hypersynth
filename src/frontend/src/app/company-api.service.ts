import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../environments/environment';

export interface Company {
  id: string;
  name: string;
  companyEmail: string;
  country: string;
  timezone: string;
  legalName: string | null;
  website: string | null;
  industry: string | null;
  companySize: string | null;
  phone: string | null;
  billingEmail: string | null;
  address: string | null;
  taxVatNumber: string | null;
  status: string;
  createdAt: string;
  updatedAt: string;
}

export interface UpdateCompanyPayload {
  name: string;
  companyEmail: string;
  country: string;
  timezone: string;
  legalName: string | null;
  website: string | null;
  industry: string | null;
  companySize: string | null;
  phone: string | null;
  billingEmail: string | null;
  address: string | null;
  taxVatNumber: string | null;
}

@Injectable({
  providedIn: 'root',
})
export class CompanyApiService {
  private readonly http = inject(HttpClient);

  getCompany(): Observable<Company> {
    const url = `${environment.apiBaseUrl}/api/v1/company`;
    return this.http.get<Company>(url);
  }

  updateCompany(payload: UpdateCompanyPayload): Observable<Company> {
    const url = `${environment.apiBaseUrl}/api/v1/company`;
    return this.http.patch<Company>(url, payload);
  }

  static errorMessage(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const body = err.error as { message?: string } | null;
      if (body && typeof body.message === 'string' && body.message.length > 0) {
        return body.message;
      }
      return `Could not save company profile (HTTP ${err.status}).`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }
}

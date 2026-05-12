import { HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject, signal } from '@angular/core';
import { Observable, catchError, map, of, tap } from 'rxjs';

import { CompanyApiService } from './company-api.service';

@Injectable({
  providedIn: 'root',
})
export class CompanyAccessService {
  private readonly companyApi = inject(CompanyApiService);
  private readonly hasCompany = signal<boolean | null>(null);

  readonly hasCompanyAssociation = this.hasCompany.asReadonly();

  resolveHasCompanyAssociation(): Observable<boolean> {
    const cached = this.hasCompany();
    if (cached !== null) {
      return of(cached);
    }

    return this.companyApi.getCompany().pipe(
      map(() => true),
      tap((hasCompany) => this.hasCompany.set(hasCompany)),
      catchError((err: unknown) => {
        if (err instanceof HttpErrorResponse && err.status === 404) {
          this.hasCompany.set(false);
          return of(false);
        }
        throw err;
      }),
    );
  }

  clearAssociationCache(): void {
    this.hasCompany.set(null);
  }
}

import { HttpErrorResponse, HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { Router } from '@angular/router';
import { catchError, throwError } from 'rxjs';

import { AuthService } from './auth.service';

function isCompanyDisabledError(err: HttpErrorResponse): boolean {
  if (err.status !== 403) {
    return false;
  }
  const body = err.error as { code?: string } | null;
  return body?.code === 'company_disabled';
}

export const companyDisabledInterceptor: HttpInterceptorFn = (req, next) => {
  const auth = inject(AuthService);
  const router = inject(Router);

  return next(req).pipe(
    catchError((err: unknown) => {
      if (err instanceof HttpErrorResponse && isCompanyDisabledError(err)) {
        auth.clearSession();
        auth.setDisabledRedirectInFlight(true);
        void router.navigateByUrl('/company-disabled');
      }
      return throwError(() => err);
    }),
  );
};

import { HttpErrorResponse, HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { Router } from '@angular/router';
import { catchError, throwError } from 'rxjs';

import { AuthService } from './auth.service';

function isUserDisabledError(err: HttpErrorResponse): boolean {
  if (err.status !== 403) {
    return false;
  }
  const body = err.error as { code?: string } | null;
  return body?.code === 'user_disabled';
}

export const userDisabledInterceptor: HttpInterceptorFn = (req, next) => {
  const auth = inject(AuthService);
  const router = inject(Router);

  return next(req).pipe(
    catchError((err: unknown) => {
      if (err instanceof HttpErrorResponse && isUserDisabledError(err)) {
        auth.clearSession();
        void router.navigate(['/login'], { queryParams: { reason: 'user_disabled' } });
      }
      return throwError(() => err);
    }),
  );
};

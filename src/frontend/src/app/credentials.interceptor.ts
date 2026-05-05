import { HttpInterceptorFn } from '@angular/common/http';

/** Sends cookies (session) on API requests to the backend. */
export const credentialsInterceptor: HttpInterceptorFn = (req, next) =>
  next(req.clone({ withCredentials: true }));

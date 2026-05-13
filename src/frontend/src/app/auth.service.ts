import { Injectable, computed, inject, signal } from '@angular/core';
import { Observable, catchError, map, of, tap } from 'rxjs';

import { AuthApiService, CurrentUser } from './auth-api.service';
import { CompanyAccessService } from './company-access.service';

@Injectable({
  providedIn: 'root',
})
export class AuthService {
  private readonly authApi = inject(AuthApiService);
  private readonly companyAccess = inject(CompanyAccessService);

  private readonly user = signal<CurrentUser | null>(null);

  readonly currentUser = this.user.asReadonly();

  readonly isCompanyUser = computed(() => this.user()?.accountType === 'company');
  readonly isCompanyAdmin = computed(() => this.user()?.role === 'company_admin');
  readonly canManageCompanyProfile = this.isCompanyAdmin;
  readonly canInviteUsers = computed(() => {
    const role = this.user()?.role;
    return role === 'company_admin' || role === 'project_manager';
  });

  /** Resolves session with GET /me when user is not cached. */
  ensureAuthenticated(): Observable<boolean> {
    if (this.user() !== null) {
      return of(true);
    }
    return this.authApi.me().pipe(
      tap((u) => this.user.set(u)),
      map(() => true),
      catchError(() => {
        this.user.set(null);
        return of(false);
      }),
    );
  }

  login(email: string, password: string): Observable<CurrentUser> {
    return this.authApi.login({ email, password }).pipe(tap((u) => this.user.set(u)));
  }

  logout(): Observable<void> {
    return this.authApi.logout().pipe(
      tap(() => {
        this.user.set(null);
        this.companyAccess.clearAssociationCache();
      }),
    );
  }
}

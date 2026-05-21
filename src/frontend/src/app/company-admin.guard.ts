import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';

import { AuthService } from './auth.service';

/** Restricts navigation to users with the Company Admin role. */
export const companyAdminGuard: CanActivateFn = () => {
  const auth = inject(AuthService);
  const router = inject(Router);

  if (auth.isCompanyAdmin()) {
    return true;
  }

  void router.navigate(['/app', '404']);
  return false;
};

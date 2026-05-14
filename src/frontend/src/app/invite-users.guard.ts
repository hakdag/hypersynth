import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';

import { AuthService } from './auth.service';

/** Company users who may invite (Company Admin or Project Manager per Phase 1 FRD). */
export const inviteUsersGuard: CanActivateFn = () => {
  const auth = inject(AuthService);
  const router = inject(Router);

  if (auth.canInviteUsers()) {
    return true;
  }

  void router.navigate(['/app', '404']);
  return false;
};

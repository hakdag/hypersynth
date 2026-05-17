import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { firstValueFrom } from 'rxjs';

import { AuthService } from './auth.service';

export const systemAdminGuard: CanActivateFn = async () => {
  const auth = inject(AuthService);
  const router = inject(Router);

  const ok = await firstValueFrom(auth.ensureAuthenticated());
  if (!ok) {
    await router.navigate(['/login']);
    return false;
  }

  if (auth.isSystemAdmin()) {
    return true;
  }

  await router.navigate(['/app/projects']);
  return false;
};

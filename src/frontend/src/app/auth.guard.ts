import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { firstValueFrom } from 'rxjs';

import { AuthService } from './auth.service';

export const authGuard: CanActivateFn = async (_route, state) => {
  const auth = inject(AuthService);
  const router = inject(Router);
  const ok = await firstValueFrom(auth.ensureAuthenticated());
  if (ok) {
    return true;
  }
  await router.navigate(['/login'], { queryParams: { returnUrl: state.url } });
  return false;
};

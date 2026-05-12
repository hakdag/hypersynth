import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { firstValueFrom } from 'rxjs';

import { CompanyAccessService } from './company-access.service';

export const companyGuard: CanActivateFn = async () => {
  const companyAccess = inject(CompanyAccessService);
  const router = inject(Router);

  try {
    const hasCompany = await firstValueFrom(companyAccess.resolveHasCompanyAssociation());
    if (hasCompany) {
      return true;
    }
  } catch {
    return false;
  }

  await router.navigate(['/app', '404']);
  return false;
};

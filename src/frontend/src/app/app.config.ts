import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import { provideHttpClient, withFetch, withInterceptors } from '@angular/common/http';
import { provideRouter } from '@angular/router';

import { companyDisabledInterceptor } from './company-disabled.interceptor';
import { credentialsInterceptor } from './credentials.interceptor';
import { userDisabledInterceptor } from './user-disabled.interceptor';
import { routes } from './app.routes';

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideHttpClient(
      withFetch(),
      withInterceptors([
        credentialsInterceptor,
        companyDisabledInterceptor,
        userDisabledInterceptor,
      ]),
    ),
    provideRouter(routes),
  ],
};

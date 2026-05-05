import { Routes } from '@angular/router';

import { AccountPlaceholder } from './account-placeholder/account-placeholder';
import { authGuard } from './auth.guard';
import { Login } from './login/login';
import { ProjectDetail } from './project-detail/project-detail';
import { ProjectList } from './project-list/project-list';
import { Register } from './register/register';
import { Shell } from './shell/shell';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'register' },
  { path: 'register', component: Register },
  { path: 'login', component: Login },
  {
    path: 'app',
    component: Shell,
    canActivate: [authGuard],
    children: [
      { path: '', pathMatch: 'full', redirectTo: 'projects' },
      { path: 'projects', component: ProjectList },
      { path: 'projects/:projectId', component: ProjectDetail },
      { path: 'account', component: AccountPlaceholder },
    ],
  },
  { path: '**', redirectTo: 'register' },
];

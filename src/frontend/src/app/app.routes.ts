import { Routes } from '@angular/router';

import { AccountPlaceholder } from './account-placeholder/account-placeholder';
import { LoginPlaceholder } from './login-placeholder/login-placeholder';
import { ProjectDetail } from './project-detail/project-detail';
import { ProjectList } from './project-list/project-list';
import { Shell } from './shell/shell';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'login' },
  { path: 'login', component: LoginPlaceholder },
  {
    path: 'app',
    component: Shell,
    children: [
      { path: '', pathMatch: 'full', redirectTo: 'projects' },
      { path: 'projects', component: ProjectList },
      { path: 'projects/:projectId', component: ProjectDetail },
      { path: 'account', component: AccountPlaceholder },
    ],
  },
  { path: '**', redirectTo: 'login' },
];

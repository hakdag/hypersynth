import { Routes } from '@angular/router';

import { AccountPlaceholder } from './account-placeholder/account-placeholder';
import { authGuard } from './auth.guard';
import { FeatureCreate } from './feature-create/feature-create';
import { FeatureDetail } from './feature-detail/feature-detail';
import { FeatureView } from './feature-view/feature-view';
import { Login } from './login/login';
import { ProjectCreate } from './project-create/project-create';
import { ProjectDetail } from './project-detail/project-detail';
import { ProjectEdit } from './project-edit/project-edit';
import { ProjectList } from './project-list/project-list';
import { Register } from './register/register';
import { Shell } from './shell/shell';
import { TaskCreate } from './task-create/task-create';
import { TaskView } from './task-view/task-view';

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
      { path: 'projects/new', component: ProjectCreate },
      { path: 'projects/:projectId/features/new', component: FeatureCreate },
      { path: 'projects/:projectId/features/:featureId/tasks/new', component: TaskCreate },
      { path: 'projects/:projectId/features/:featureId/tasks/:taskId', component: TaskView },
      { path: 'projects/:projectId/features/:featureId/edit', component: FeatureDetail },
      { path: 'projects/:projectId/features/:featureId', component: FeatureView },
      { path: 'projects/:projectId/edit', component: ProjectEdit },
      { path: 'projects/:projectId', component: ProjectDetail },
      { path: 'account', component: AccountPlaceholder },
    ],
  },
  { path: '**', redirectTo: 'register' },
];

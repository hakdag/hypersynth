import { Routes } from '@angular/router';

import { AccountPlaceholder } from './account-placeholder/account-placeholder';
import { authGuard } from './auth.guard';
import { companyGuard } from './company.guard';
import { CompanyProfile } from './company-profile/company-profile';
import { InvitationAccept } from './invitation-accept/invitation-accept';
import { InvitationCreate } from './invitation-create/invitation-create';
import { InvitationList } from './invitation-list/invitation-list';
import { inviteUsersGuard } from './invite-users.guard';
import { FeatureCreate } from './feature-create/feature-create';
import { FeatureDetail } from './feature-detail/feature-detail';
import { FeatureView } from './feature-view/feature-view';
import { AdminCompaniesList } from './admin-companies-list/admin-companies-list';
import { AdminCompanyDetail } from './admin-company-detail/admin-company-detail';
import { AdminUserDetail } from './admin-user-detail/admin-user-detail';
import { AdminAiUsage } from './admin-ai-usage/admin-ai-usage';
import { AdminUsersList } from './admin-users-list/admin-users-list';
import { CompanyDisabled } from './company-disabled/company-disabled';
import { Login } from './login/login';
import { SystemAdminDashboard } from './system-admin-dashboard/system-admin-dashboard';
import { systemAdminGuard } from './system-admin.guard';
import { NotFound } from './not-found/not-found';
import { ProjectCreate } from './project-create/project-create';
import { ProjectDetail } from './project-detail/project-detail';
import { ProjectEdit } from './project-edit/project-edit';
import { ProjectList } from './project-list/project-list';
import { Register } from './register/register';
import { Shell } from './shell/shell';
import { TaskCreate } from './task-create/task-create';
import { TaskEdit } from './task-edit/task-edit';
import { TaskView } from './task-view/task-view';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'register' },
  { path: 'register', component: Register },
  { path: 'login', component: Login },
  { path: 'company-disabled', component: CompanyDisabled },
  { path: 'invitations/accept', component: InvitationAccept },
  {
    path: 'app',
    component: Shell,
    canActivate: [authGuard],
    children: [
      { path: '', pathMatch: 'full', redirectTo: 'projects' },
      {
        path: 'admin',
        component: SystemAdminDashboard,
        canActivate: [systemAdminGuard],
        children: [
          { path: '', pathMatch: 'full', redirectTo: 'companies' },
          { path: 'companies', component: AdminCompaniesList },
          { path: 'companies/:companyId', component: AdminCompanyDetail },
          { path: 'users', component: AdminUsersList },
          { path: 'users/:userId', component: AdminUserDetail },
          { path: 'ai-usage', component: AdminAiUsage },
        ],
      },
      { path: 'projects', component: ProjectList },
      { path: 'projects/new', component: ProjectCreate },
      { path: 'projects/:projectId/features/new', component: FeatureCreate },
      { path: 'projects/:projectId/features/:featureId/tasks/new', component: TaskCreate },
      {
        path: 'projects/:projectId/features/:featureId/tasks/:taskId/edit',
        component: TaskEdit,
      },
      { path: 'projects/:projectId/features/:featureId/tasks/:taskId', component: TaskView },
      { path: 'projects/:projectId/features/:featureId/edit', component: FeatureDetail },
      { path: 'projects/:projectId/features/:featureId', component: FeatureView },
      { path: 'projects/:projectId/edit', component: ProjectEdit },
      { path: 'projects/:projectId', component: ProjectDetail },
      { path: 'company', component: CompanyProfile, canActivate: [companyGuard] },
      {
        path: 'team/invitations',
        component: InvitationList,
        canActivate: [companyGuard, inviteUsersGuard],
      },
      {
        path: 'team/invitations/new',
        component: InvitationCreate,
        canActivate: [companyGuard, inviteUsersGuard],
      },
      { path: '404', component: NotFound },
      { path: 'account', component: AccountPlaceholder },
    ],
  },
  { path: '**', redirectTo: 'register' },
];

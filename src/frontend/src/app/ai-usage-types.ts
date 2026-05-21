export type AiUsageSort = 'tokens' | 'cost';

export interface AiUsageTotals {
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface AiUsageByUserRow {
  userId: string;
  userEmail: string;
  userFullName: string;
  companyId: string | null;
  companyName: string | null;
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface AiUsageByProviderModelRow {
  provider: string;
  model: string;
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface AiUsageFailureRow {
  id: string;
  companyId: string | null;
  companyName: string | null;
  userId: string;
  userEmail: string;
  provider: string;
  model: string;
  operationType: string;
  errorCode: string | null;
  createdAt: string;
}

export interface AiUsageDateRange {
  from?: string;
  to?: string;
}

export interface AiUsageListOptions extends AiUsageDateRange {
  limit?: number;
  offset?: number;
}

export interface CompanyAiUsageByUserRow {
  userId: string;
  userEmail: string;
  userFullName: string;
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface CompanyAiUsageByProjectRow {
  projectId: string | null;
  projectName: string | null;
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  successCount: number;
  failureCount: number;
}

export interface CompanyAiUsageFailureRow {
  id: string;
  userId: string;
  userEmail: string;
  provider: string;
  model: string;
  operationType: string;
  errorCode: string | null;
  createdAt: string;
}

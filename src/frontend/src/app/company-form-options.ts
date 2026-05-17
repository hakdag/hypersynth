export const COUNTRY_OPTIONS = [
  'United States',
  'United Kingdom',
  'Canada',
  'Germany',
  'France',
  'Netherlands',
  'Australia',
  'India',
  'Japan',
  'Other',
] as const;

const FALLBACK_TIMEZONES = [
  'UTC',
  'America/New_York',
  'America/Chicago',
  'America/Denver',
  'America/Los_Angeles',
  'Europe/London',
  'Europe/Berlin',
  'Europe/Paris',
  'Asia/Tokyo',
  'Asia/Kolkata',
  'Australia/Sydney',
] as const;

export function buildTimezoneOptions(): readonly string[] {
  const supported = Intl as typeof Intl & {
    supportedValuesOf?: (key: 'timeZone') => string[];
  };
  if (typeof supported.supportedValuesOf === 'function') {
    return supported.supportedValuesOf('timeZone');
  }
  return FALLBACK_TIMEZONES;
}

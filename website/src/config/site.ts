export const repositoryUrl = 'https://github.com/vgerbot-libraries/vgui';

export const releasesUrl = `${repositoryUrl}/releases`;
export const issuesUrl = `${repositoryUrl}/issues`;

export interface DocVersion {
  label: string;
  href?: string;
  current?: boolean;
}

// Add historical snapshots only after their versioned site is actually deployed.
export const versions: DocVersion[] = [{ label: 'v0.1.0', current: true }];

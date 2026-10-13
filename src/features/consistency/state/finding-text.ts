import type { Finding, Rule } from '../api/consistency-api';

/** The dictionary key of a rule's one-line title. */
export function titleKey(rule: Rule): `consistency.rule_${Rule}` {
  return `consistency.rule_${rule}`;
}

/** The dictionary key of a rule's explanation. */
export function explanationKey(rule: Rule): `consistency.explain_${Rule}` {
  return `consistency.explain_${rule}`;
}

/** Values the explanation of a finding interpolates: its facts and the names it involves. */
export function explanationParams(finding: Finding): Record<string, string> {
  return {
    ...finding.facts,
    subject: finding.subject.name,
    related: finding.related.map((entity) => entity.name).join(', '),
  };
}

/** Keeps the findings matching the severity and rule filters; an empty filter keeps all. */
export function filterFindings(
  findings: readonly Finding[],
  severity: string,
  rule: string,
): Finding[] {
  return findings.filter(
    (finding) =>
      (severity === '' || finding.severity === severity) && (rule === '' || finding.rule === rule),
  );
}

import { commonTexts } from './common';
import { shellTexts } from './shell';
import { projectTexts } from './project';
import { defaultTexts } from './defaults';
import { entityTexts } from './entity';
import { catalogTexts } from './catalog';
import { relationshipTexts } from './relationship';
import { structureTexts } from './structure';
import { libraryTexts } from './library';
import { graphTexts } from './graph';
import { matrixTexts } from './matrix';
import { timelineTexts } from './timeline';
import { consistencyTexts } from './consistency';
import { dossierTexts } from './dossier';
import { historyTexts } from './history';
import { paletteTexts } from './palette';
import { dataTexts } from './data';
import { onboardingTexts } from './onboarding';
import { assistantTexts } from './assistant';

/** Every group of texts, one per area of the interface. A section lives in one group only. */
export const groups = [
  defaultTexts,
  commonTexts,
  shellTexts,
  projectTexts,
  entityTexts,
  catalogTexts,
  relationshipTexts,
  structureTexts,
  libraryTexts,
  graphTexts,
  matrixTexts,
  timelineTexts,
  consistencyTexts,
  dossierTexts,
  historyTexts,
  paletteTexts,
  dataTexts,
  onboardingTexts,
  assistantTexts,
] as const;

export type Texts = typeof defaultTexts &
  typeof commonTexts &
  typeof shellTexts &
  typeof projectTexts &
  typeof entityTexts &
  typeof catalogTexts &
  typeof relationshipTexts &
  typeof structureTexts &
  typeof libraryTexts &
  typeof graphTexts &
  typeof matrixTexts &
  typeof timelineTexts &
  typeof consistencyTexts &
  typeof dossierTexts &
  typeof historyTexts &
  typeof paletteTexts &
  typeof dataTexts &
  typeof onboardingTexts &
  typeof assistantTexts;

import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
  i18n: defineCollection({
    loader: i18nLoader(),
    schema: i18nSchema({
      extend: z.object({
        'nav.docs': z.string(),
        'nav.components': z.string(),
        'nav.resources': z.string(),
        'nav.apiDoc': z.string(),
        'nav.releases': z.string(),
        'nav.issues': z.string(),
        'nav.version': z.string(),
        'nav.latest': z.string(),
        'nav.current': z.string(),
        'nav.primary': z.string(),
        'nav.external': z.string(),
      }),
    }),
  }),
};

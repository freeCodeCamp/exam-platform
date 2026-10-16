import { defineCollection } from "astro:content";
import { file } from "astro/loaders";

import { z } from "astro/zod";

const questionSchema = z.object({
  passage: z.string(),
  questions: z.array(
    z.object({
      prompt: z.string(),
      options: z.array(z.string()),
      correct: z.number(),
    }),
  ),
});

const questions = defineCollection({
  loader: file("client/exams/sample.yaml"),

  schema: questionSchema,
});

export const collections = { questions };

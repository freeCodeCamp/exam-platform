import { defineCollection } from "astro:content";
import { file } from "astro/loaders";

import { z } from "astro/zod";

const questionSchema = z.object({
  passage: z.string(),
  questions: z.array(
    z.object({
      prompt: z.string(),
      options: z
        .array(
          z.object({
            value: z.string(),
            id: z.string(),
          }),
        )
        .refine(
          (xs) => {
            const values = xs.map(({ value }) => value);
            return new Set(values).size === xs.length;
          },
          { message: "Duplicate options are not allowed" },
        ),
      correct: z.number(),
      id: z.string(),
    }),
  ),
});

const questions = defineCollection({
  loader: file("client/exams/sample.yaml"),

  schema: questionSchema,
});

export const collections = { questions };

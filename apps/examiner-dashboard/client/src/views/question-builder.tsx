import { Text } from "@chakra-ui/react";

import type { CollectionEntry } from 'astro:content';

import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { MultipleChoice } from "@/components/questions/multiple-choice";

type Question = CollectionEntry<'questions'>['data']

export function QuestionBuilder({ question }: { question: Question }) {
  return (
    <AppRoot>
      <PageShell title="Question Builder">
        <Text>Author exams.</Text>

        {question.questions.map(({ options, prompt, id }) =>
          <MultipleChoice key={id} options={options} prompt={prompt} />



        )}

        {question.passage}
      </PageShell>
    </AppRoot>
  );
}

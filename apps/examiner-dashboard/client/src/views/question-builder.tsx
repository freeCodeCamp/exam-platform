import { Text } from "@chakra-ui/react";
import { DragDropProvider } from '@dnd-kit/react';
import { useState } from "react";

import type { CollectionEntry } from 'astro:content';

import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { MultipleChoice } from "@/components/questions/multiple-choice";
import { Draggable } from "@/components/ui/draggable";
import { Droppable } from "@/components/ui/droppable";

type Question = CollectionEntry<'questions'>['data']

export function QuestionBuilder({ question }: { question: Question }) {
  const [isDropped, setIsDropped] = useState(false);
  return (
    <AppRoot>
      <PageShell title="Question Builder">
        <Text>Author exams.</Text>
        <DragDropProvider
          onDragEnd={(event) => {
            if (event.canceled) return;

            const { target } = event.operation;
            setIsDropped(target?.id === 'droppable');
          }}
        >
        Place for dragging
          {!isDropped && <Draggable />}

          <Droppable id="droppable">
            {isDropped && <Draggable />}
          </Droppable>
        </DragDropProvider>
        {question.questions.map(({ options, prompt, id }) =>
          <MultipleChoice key={id} options={options} prompt={prompt} />



        )}

        {question.passage}
      </PageShell>
    </AppRoot>
  );
}

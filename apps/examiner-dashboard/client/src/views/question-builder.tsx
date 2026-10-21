import { Text } from "@chakra-ui/react";
import { DragDropProvider } from "@dnd-kit/react";
import { useState } from "react";

import type { CollectionEntry } from "astro:content";

import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { MultipleChoice } from "@/components/questions/multiple-choice";
import { Draggable } from "@/components/ui/draggable";
import { Droppable } from "@/components/ui/droppable";

type Question = CollectionEntry<"questions">["data"];

interface HasId {
  id: number;
}

interface Form extends HasId {}
interface Other extends HasId {}

const max: (xs: number[]) => number = (xs) => xs.reduce((prev, curr) => Math.max(prev, curr), 0);

export function QuestionBuilder({ question }: { question: Question }) {
  const [forms, setForms] = useState<Form[]>([]);
  const [others, setOthers] = useState<Other[]>([]);
  return (
    <AppRoot>
      <PageShell title="Question Builder">
        <Text>Author exams.</Text>
        <DragDropProvider
          onDragEnd={(event) => {
            if (event.canceled) return;

            const { target, source } = event.operation;

            if (source?.id == "form") {
              const maxId = max(forms.map(({ id }) => id));
              setForms((xs) => [...xs, { id: maxId + 1 }]);
            } else {
              const maxId = max(others.map(({ id }) => id));
              setOthers((xs) => [...xs, { id: maxId + 1 }]);
            }
          }}
        >
          Place for dragging
          <Draggable id="form" />
          <Draggable id="other" />
          <Droppable id="droppable">drop here</Droppable>
        </DragDropProvider>

        {forms.map(({ id }) => (
          <div key={id}> Form: {id}</div>
        ))}
        {others.map(({ id }) => (
          <div key={id}> Other: {id}</div>
        ))}

        {question.questions.map(({ options, prompt, id }) => (
          <MultipleChoice key={id} options={options} prompt={prompt} />
        ))}

        {question.passage}
      </PageShell>
    </AppRoot>
  );
}

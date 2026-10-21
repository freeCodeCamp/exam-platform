import { Flex, Text, VStack } from "@chakra-ui/react";
import { DragDropProvider } from "@dnd-kit/react";
import { useState } from "react";

import type { CollectionEntry } from "astro:content";

import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { Draggable } from "@/components/ui/draggable";
import { Droppable } from "@/components/ui/droppable";

type Question = CollectionEntry<"questions">["data"];

interface HasId {
  id: number;
}

interface Form extends HasId {}
interface Other extends HasId {}

const max: (xs: number[]) => number = (xs) => xs.reduce((prev, curr) => Math.max(prev, curr), 0);

function SideBar() {
  return (
    <VStack border="md" flexGrow={0.5}>
      <Draggable id="form" />
      <Draggable id="other" />
    </VStack>
  );
}

function Preview({ forms, others }: { forms: Form[]; others: Other[] }) {
  return (
    <>
      Preview:
      {forms.map(({ id }) => (
        <div key={id}> Form: {id}</div>
      ))}
      {others.map(({ id }) => (
        <div key={id}> Other: {id}</div>
      ))}{" "}
    </>
  );
}

export function QuestionBuilder() {
  const [forms, setForms] = useState<Form[]>([]);
  const [others, setOthers] = useState<Other[]>([]);
  return (
    <AppRoot>
      <PageShell title="Question Builder">
        <Text>Author exams.</Text>
        <DragDropProvider
          onDragEnd={(event) => {
            if (event.canceled) return;

            const { source } = event.operation;

            if (source?.id == "form") {
              const maxId = max(forms.map(({ id }) => id));
              setForms((xs) => [...xs, { id: maxId + 1 }]);
            } else {
              const maxId = max(others.map(({ id }) => id));
              setOthers((xs) => [...xs, { id: maxId + 1 }]);
            }
          }}
        >
          <Flex justify={"space-between"}>
            <Droppable id="droppable">
              <Preview forms={forms} others={others} />
            </Droppable>
            <SideBar />
          </Flex>
        </DragDropProvider>
      </PageShell>
    </AppRoot>
  );
}

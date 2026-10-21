import { Flex, Text, VStack } from "@chakra-ui/react";
import { DragDropProvider } from "@dnd-kit/react";
import { Fragment, useState } from "react";

import type { CollectionEntry } from "astro:content";

import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { Draggable } from "@/components/ui/draggable";
import { Droppable } from "@/components/ui/droppable";
import { MultipleChoice } from "@/components/questions/multiple-choice";

type Question = CollectionEntry<"questions">["data"]["questions"][number];

interface HasId {
  id: string;
}

type Paragraph = HasId & {
  text: string;
};
type MCQ = HasId & Question;

function SideBar() {
  return (
    <VStack border="md" flexGrow={0.5}>
      <Draggable id="paragraph" />
      <Draggable id="multipleChoice" />
    </VStack>
  );
}

function Preview({
  paragraphs,
  multipleChoices,
}: {
  paragraphs: Paragraph[];
  multipleChoices: MCQ[];
}) {
  return (
    <>
      Preview:
      {paragraphs.map((x) => (
        <div key={x.id}>
          {" "}
          Paragraph, id: {x.id}, text: {x.text}
        </div>
      ))}
      {multipleChoices.map((x) => (
        <Fragment key={x.id}>
          MCQ: {x.id}
          <MultipleChoice prompt={x.prompt} options={x.options} />
        </Fragment>
      ))}{" "}
    </>
  );
}

export function QuestionBuilder() {
  const [paragraphs, setparagraphs] = useState<Paragraph[]>([]);
  const [multipleChoices, setMultipleChoices] = useState<MCQ[]>([]);
  return (
    <AppRoot>
      <PageShell title="Question Builder">
        <Text>Author exams.</Text>
        <DragDropProvider
          onDragEnd={(event) => {
            if (event.canceled) return;

            const { source } = event.operation;

            if (source?.id == "paragraph") {
              const id = crypto.randomUUID();
              setparagraphs((xs) => [...xs, { id, text: "" }]);
            } else {
              const id = crypto.randomUUID();
              setMultipleChoices((xs) => [...xs, { id, prompt: "", correct: 0, options: [] }]);
            }
          }}
        >
          <Flex justify={"space-between"}>
            <Droppable id="droppable">
              <Preview paragraphs={paragraphs} multipleChoices={multipleChoices} />
            </Droppable>
            <SideBar />
          </Flex>
        </DragDropProvider>
      </PageShell>
    </AppRoot>
  );
}

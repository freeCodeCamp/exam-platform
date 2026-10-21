import { Box, Flex, Text, VStack } from "@chakra-ui/react";
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
  kind: "paragraph";
};
type MCQ = HasId &
  Question & {
    kind: "multiple-choice";
  };

type Element = Paragraph | MCQ;

function SideBar({ elements }: { elements: Element[] }) {
  return (
    <VStack border="md" flexGrow={0.5}>
      <Draggable id="paragraph" />
      <Draggable id="multipleChoice" />
      <Box>
        Page:
        <VStack>
          {elements.map((el, id) => (
            <div key={el.id}>
              {id} {el.kind}
            </div>
          ))}
        </VStack>
      </Box>
    </VStack>
  );
}

function Preview({ elements }: { elements: Element[] }) {
  return (
    <>
      Preview:
      {elements.map((x) =>
        x.kind == "paragraph" ? (
          <div key={x.id}>
            Paragraph, id: {x.id}, text: {x.text}
          </div>
        ) : (
          <Fragment key={x.id}>
            MCQ: {x.id}
            <MultipleChoice prompt={x.prompt} options={x.options} />
          </Fragment>
        ),
      )}
    </>
  );
}

export function QuestionBuilder() {
  const [elements, setElements] = useState<Element[]>([]);
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
              setElements((xs) => [...xs, { id, text: "", kind: "paragraph" }]);
            } else {
              const id = crypto.randomUUID();
              setElements((xs) => [
                ...xs,
                { id, prompt: "", correct: 0, options: [], kind: "multiple-choice" },
              ]);
            }
          }}
        >
          <Flex justify={"space-between"}>
            <Droppable id="droppable">
              <Preview elements={elements} />
            </Droppable>
            <SideBar elements={elements} />
          </Flex>
        </DragDropProvider>
      </PageShell>
    </AppRoot>
  );
}

import { Box, Flex, Text, VStack } from "@chakra-ui/react";
import { DragDropProvider } from "@dnd-kit/react";
import { Fragment, useState } from "react";
import { useForm, type SubmitHandler } from "react-hook-form";

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

interface ParagraphInputs {
  text: string;
}

function ParagraphEditor({
  paragraph,
  updateParagraph,
}: {
  paragraph: Paragraph;
  updateParagraph: (id: string, text: string) => void;
}) {
  const { register, handleSubmit } = useForm<ParagraphInputs>();
  const onSubmit = handleSubmit((data) => updateParagraph(paragraph.id, data.text));

  return (
    <div>
      {paragraph.id} {paragraph.kind}
      <form onSubmit={onSubmit}>
        <label htmlFor="para">Text:</label>

        <input {...register("text")} />
      </form>
    </div>
  );
}

type UpdateParagraph = (id: string, text: string) => void;

function SideBar({
  elements,
  updateParagraph,
}: {
  elements: Element[];
  updateParagraph: UpdateParagraph;
}) {
  return (
    <VStack border="md" flexGrow={0.5}>
      <Draggable id="paragraph" />
      <Draggable id="multipleChoice" />
      <Box>
        Page:
        <VStack>
          {elements.map((el, id) =>
            el.kind !== "paragraph" ? (
              <div key={el.id}>
                {id} {el.kind}
              </div>
            ) : (
              <ParagraphEditor
                key={id}
                paragraph={el}
                updateParagraph={(id, text) => updateParagraph(id, text)}
              />
            ),
          )}
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
  const updateParagraph: UpdateParagraph = (id, text) => {
    setElements((xs) => xs.map((x) => (x.id === id ? { id, text, kind: "paragraph" } : x)));
  };

  return (
    <AppRoot>
      <PageShell title="Question Builder">
        <Text>Author exams.</Text>
        <DragDropProvider
          onDragEnd={(event) => {
            // e.g. if the user presses Escape
            if (event.canceled) return;
            // if the user drops somewhere other than the preview
            if (!event.operation.target) return;

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
            <SideBar elements={elements} updateParagraph={updateParagraph} />
          </Flex>
        </DragDropProvider>
      </PageShell>
    </AppRoot>
  );
}

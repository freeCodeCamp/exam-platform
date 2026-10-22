import { Box, Button, Flex, Text, VStack } from "@chakra-ui/react";
import { DragDropProvider } from "@dnd-kit/react";
import { Fragment, useState } from "react";
import { useForm } from "react-hook-form";

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
type MCQ = Question & {
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

function MultipleChoiceEditor({
  mcq,
  updateMCQ,
}: {
  mcq: MCQ;
  updateMCQ: (data: Question) => void;
}) {
  const { register, handleSubmit } = useForm<Omit<Question, "id">>({ defaultValues: mcq });
  const onSubmit = handleSubmit((data) => updateMCQ({ ...data, id: mcq.id }));

  const addOption = () =>
    updateMCQ({
      ...mcq,
      options: [
        ...mcq.options,
        { value: "Option: " + mcq.options.length, id: crypto.randomUUID() },
      ],
    });

  return (
    <div>
      {mcq.id} {mcq.kind}
      <form onSubmit={onSubmit}>
        <label>Prompt:</label>
        <input {...register("prompt")} />
      </form>
      <Button onClick={addOption}> Add option </Button>
    </div>
  );
}

type UpdateParagraph = (id: string, text: string) => void;
type UpdateMCQ = (data: Question) => void;

function SideBar({
  elements,
  updateParagraph,
  updateMCQ,
}: {
  elements: Element[];
  updateParagraph: UpdateParagraph;
  updateMCQ: UpdateMCQ;
}) {
  return (
    <VStack border="md" flexGrow={0.5}>
      <Draggable id="paragraph" />
      <Draggable id="multipleChoice" />
      <Box>
        Page:
        <VStack>
          {elements.map((el) =>
            el.kind !== "paragraph" ? (
              <MultipleChoiceEditor key={el.id} mcq={el} updateMCQ={updateMCQ} />
            ) : (
              <ParagraphEditor
                key={el.id}
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

  const updateMCQ: UpdateMCQ = (data) =>
    setElements((xs) =>
      xs.map((x) => (x.id === data.id ? { ...data, kind: "multiple-choice" } : x)),
    );

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
              setElements((xs) => [
                ...xs,
                { id, text: "Placeholder paragraph", kind: "paragraph" },
              ]);
            } else {
              const id = crypto.randomUUID();
              setElements((xs) => [
                ...xs,
                {
                  id,
                  prompt: "Placeholder prompt",
                  correct: 0,
                  options: [],
                  kind: "multiple-choice",
                },
              ]);
            }
          }}
        >
          <Flex justify={"space-between"}>
            <Droppable id="droppable">
              <Preview elements={elements} />
            </Droppable>
            <SideBar elements={elements} updateParagraph={updateParagraph} updateMCQ={updateMCQ} />
          </Flex>
        </DragDropProvider>
      </PageShell>
    </AppRoot>
  );
}

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

interface Text extends HasId {
  value: string;
};
type Paragraph = Text & {
  kind: "paragraph";
};
type MCQ = Question & {
  kind: "multiple-choice";
};

type Option = Question["options"][number];

type Element = Paragraph | MCQ;

function MultipleChoiceEditor({
  mcq,
  updateMCQ,
}: {
  mcq: MCQ;
  updateMCQ: (data: Question) => void;
}) {
  const addOption = () =>
    updateMCQ({
      ...mcq,
      options: [
        ...mcq.options,
        { value: "Option: " + mcq.options.length, id: crypto.randomUUID() },
      ],
    });

  const updateOption = (data: Option) =>
    updateMCQ({ ...mcq, options: mcq.options.map((opt) => (opt.id == data.id ? data : opt)) });

  const updatePrompt: (text: Text) => void = (text) => updateMCQ({ ...mcq, prompt: text.value });

  return (
    <div>
      {mcq.id} {mcq.kind}
      <TextEditor
        label="Prompt: "
        text={{ id: mcq.id, value: mcq.prompt }}
        updateText={updatePrompt}
      />
      {mcq.options.map((opt) => (
        <TextEditor key={opt.id} label="Option: " text={opt} updateText={updateOption} />
      ))}
      <Button onClick={addOption}> Add option </Button>
    </div>
  );
}


function TextEditor({
  label,
  text,
  updateText,
}: {
  label: string;
  text: Text;
  updateText: (text: Text) => void;
}) {
  const { register, handleSubmit } = useForm<Option>({ defaultValues: text });
  const onSubmit = handleSubmit(updateText);

  return (
    <form onSubmit={onSubmit}>
      <label>
        {label} {text.id}
      </label>
      <input {...register("value")} />
    </form>
  );
}

type UpdateParagraph = (text: Text) => void;
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
<TextEditor key={el.id} label="Paragraph: " text={el} updateText={updateParagraph}/>
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
            Paragraph, id: {x.id}, text: {x.value}
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
  const updateParagraph: UpdateParagraph = (text) => {
    setElements((xs) => xs.map((x) => (x.id === text.id ? { ...text, kind: "paragraph" } : x)));
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
                { id, value: "Placeholder paragraph", kind: "paragraph" },
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

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
}
type Paragraph = Text & {
  kind: "paragraph";
};
type MCQ = Question & {
  kind: "multiple-choice";
};

type Option = Question["options"][number];

type Element = Paragraph | MCQ;
type Kind = Element["kind"];

function MultipleChoiceEditor({
  element,
  updateElement,
}: {
  element: MCQ;
  updateElement: UpdateElement;
}) {
  const addOption = () =>
    updateElement({
      ...element,
      options: [
        ...element.options,
        { value: "Option: " + element.options.length, id: crypto.randomUUID() },
      ],
    });

  const updateOption = (data: Option) =>
    updateElement({
      ...element,
      options: element.options.map((opt) => (opt.id == data.id ? data : opt)),
    });

  const updatePrompt: (text: Text) => void = (text) =>
    updateElement({ ...element, prompt: text.value });

  return (
    <div>
      {element.id} {element.kind}
      <TextEditor
        label="Prompt: "
        text={{ id: element.id, value: element.prompt }}
        updateText={updatePrompt}
      />
      {element.options.map((opt) => (
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
  const { register, handleSubmit, getFieldState } = useForm<Option>({ defaultValues: text });
  const onSubmit = handleSubmit(updateText);
  const onBlur: React.FocusEventHandler = (e) => {
    if (!getFieldState("value").invalid) {
      onSubmit(e);
    }
  };

  return (
    <form onBlur={onBlur} onSubmit={onSubmit}>
      <label>
        {label} {text.id}
      </label>
      <input {...register("value", { required: true })} />
    </form>
  );
}

function ParagraphEditor({
  element,
  updateElement,
}: {
  element: Paragraph;
  updateElement: UpdateElement;
}) {
  const updateText = (text: Text) => updateElement({ ...text, kind: "paragraph" });

  return <TextEditor label="Paragraph: " text={element} updateText={updateText} />;
}

type UpdateElement = (el: Element) => void;

function EnumeratedElement({ n, kind }: { n: number; kind: Kind }) {
  const paddedNumber = n < 10 ? "0" + n : "" + n;

  return (
    <>
      {paddedNumber} {kind === "paragraph" ? "Paragraph" : "Multiple Choice"}
    </>
  );
}

function SideBar({
  elements,
  updateElement,
}: {
  elements: Element[];
  updateElement: UpdateElement;
}) {
  return (
    <VStack border="md" flexGrow={0.5}>
      <Draggable id="paragraph" />
      <Draggable id="multipleChoice" />
      <Box>
        Page:
        <VStack>
          {elements.map((el, id) => (
            <Fragment key={el.id}>
              <EnumeratedElement n={id + 1} kind={el.kind} />
              {el.kind === "multiple-choice" ? (
                <MultipleChoiceEditor element={el} updateElement={updateElement} />
              ) : (
                <ParagraphEditor element={el} updateElement={updateElement} />
              )}
            </Fragment>
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

  const updateElement: UpdateElement = (data) =>
    setElements((xs) => xs.map((x) => (x.id === data.id ? data : x)));

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
            <SideBar elements={elements} updateElement={updateElement} />
          </Flex>
        </DragDropProvider>
      </PageShell>
    </AppRoot>
  );
}

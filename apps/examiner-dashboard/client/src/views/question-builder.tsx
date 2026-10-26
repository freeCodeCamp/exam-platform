import { Box, Button, Flex, Text, VStack, Field, Input, CloseButton, Card } from "@chakra-ui/react";
import { DragDropProvider } from "@dnd-kit/react";
import { useState } from "react";
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

const kinds: Kind[] = ["paragraph", "multiple-choice"];

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

  const deleteOption = ({ id }: { id: string }) =>
    updateElement({
      ...element,
      options: element.options.filter((opt) => opt.id !== id),
    });

  const updatePrompt: (text: Text) => void = (text) =>
    updateElement({ ...element, prompt: text.value });

  return (
    <div>
      <TextEditor
        label="Prompt: "
        text={{ id: element.id, value: element.prompt }}
        updateText={updatePrompt}
      />

      {element.options.length > 0 ? <Text>Options</Text> : null}
      {element.options.map((opt, id) => (
        <TextEditor
          onDelete={deleteOption}
          key={opt.id}
          label={"" + id}
          text={opt}
          updateText={updateOption}
          options={{ orientation: "horizontal" }}
        />
      ))}
      <Button onClick={addOption}> Add option </Button>
    </div>
  );
}

function TextEditor({
  label,
  text,
  updateText,
  onDelete,
  options,
}: {
  label: string;
  text: Text;
  updateText: (text: Text) => void;
  onDelete?: (x: { id: string }) => void;
  options?: {
    orientation?: "horizontal";
  };
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
      <Field.Root {...options}>
        <Field.Label>{label}</Field.Label>
        <Input {...register("value", { required: true })} />
        {onDelete && <CloseButton onClick={() => onDelete(text)} />}
      </Field.Root>
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

  return <TextEditor label="Content" text={element} updateText={updateText} />;
}

type UpdateElement = (el: Element) => void;
type DeleteElement = (el: { id: string }) => void;

function EnumeratedElement({ n, kind }: { n: number; kind: Kind }) {
  const paddedNumber = n < 10 ? "0" + n : "" + n;

  return (
    <Text alignContent={"center"}>
      {paddedNumber} {kind === "paragraph" ? "Paragraph" : "Multiple Choice"}
    </Text>
  );
}

function SideBar({
  deleteElement,
  elements,
  updateElement,
}: {
  deleteElement: DeleteElement;
  elements: Element[];
  updateElement: UpdateElement;
}) {
  return (
    <VStack border="md" flexGrow={0.5}>
      <Draggable id="paragraph" />
      <Draggable id="multiple-choice" />
      <Box>
        Page:
        {elements.map((el, id) => (
          <Card.Root key={el.id}>
            <Card.Header>
              <Flex justifyContent={"space-between"}>
                <EnumeratedElement n={id + 1} kind={el.kind} />{" "}
                <CloseButton onClick={() => deleteElement(el)} />
              </Flex>
            </Card.Header>
            <Card.Body>
              {el.kind === "multiple-choice" ? (
                <MultipleChoiceEditor element={el} updateElement={updateElement} />
              ) : (
                <ParagraphEditor element={el} updateElement={updateElement} />
              )}
            </Card.Body>
          </Card.Root>
        ))}
      </Box>
    </VStack>
  );
}

function Preview({ elements }: { elements: Element[] }) {
  return (
    <>
      Preview:
      {elements.map((el) =>
        el.kind == "paragraph" ? (
          <p key={el.id}>{el.value}</p>
        ) : (
          <MultipleChoice key={el.id} prompt={el.prompt} options={el.options} />
        ),
      )}
    </>
  );
}

const paragraphElement = (id: string): Paragraph => ({
  id,
  value: "Placeholder paragraph",
  kind: "paragraph",
});
const multipleChoiceElement = (id: string): MCQ => ({
  id,
  prompt: "Placeholder prompt",
  correct: 0,
  options: [],
  kind: "multiple-choice",
});

const newElement = (id: string, kind: Kind): Element => {
  switch (kind) {
    case "paragraph":
      return paragraphElement(id);
    case "multiple-choice":
      return multipleChoiceElement(id);
  }
};

function isKind(x?: unknown): x is Kind {
  return kinds.includes(x as Kind);
}

export function QuestionBuilder() {
  const [elements, setElements] = useState<Element[]>([]);

  const updateElement: UpdateElement = (data) =>
    setElements((xs) => xs.map((x) => (x.id === data.id ? data : x)));

  const deleteElement = (el: { id: string }) =>
    setElements((xs) => xs.filter((x) => x.id !== el.id));

  const createElement = (kind: Kind) =>
    setElements((xs) => {
      const id = crypto.randomUUID();
      return [...xs, newElement(id, kind)];
    });

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
            const kind = source?.id;

            if (!isKind(kind)) throw Error("invalid element type: " + kind);

            createElement(kind);
          }}
        >
          <Flex justify={"space-between"}>
            <Droppable id="droppable">
              <Preview elements={elements} />
            </Droppable>
            <SideBar
              deleteElement={deleteElement}
              elements={elements}
              updateElement={updateElement}
            />
          </Flex>
        </DragDropProvider>
      </PageShell>
    </AppRoot>
  );
}

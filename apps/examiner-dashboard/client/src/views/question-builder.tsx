import {
  Box,
  Button,
  Center,
  Flex,
  Heading,
  HStack,
  Text,
  VStack,
  Field,
  Input,
  CloseButton,
  Card,
} from "@chakra-ui/react";
import { DragDropProvider } from "@dnd-kit/react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import type { IconType } from "react-icons";
import {
  LuGripVertical,
  LuListChecks,
  LuMousePointerClick,
  LuPilcrow,
} from "react-icons/lu";

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
type MultipleChoiceQuestion = Question & {
  kind: "multiple-choice";
};

type Option = Question["options"][number];

type Element = Paragraph | MultipleChoiceQuestion;
type Kind = Element["kind"];

const kinds: Kind[] = ["paragraph", "multiple-choice"];

// TODO: Move to own module
const kindMeta: Record<
  Kind,
  { label: string; description: string; icon: IconType; palette: string }
> = {
  paragraph: {
    label: "Paragraph",
    description: "Text shown to the candidate",
    icon: LuPilcrow,
    palette: "blue",
  },
  "multiple-choice": {
    label: "Multiple Choice",
    description: "Prompt with selectable options",
    icon: LuListChecks,
    palette: "purple",
  },
};

function MultipleChoiceEditor({
  element,
  updateElement,
}: {
  element: MultipleChoiceQuestion;
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
  const { register, handleSubmit, getFieldState } = useForm<Option>({
    defaultValues: text,
  });
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
  const updateText = (text: Text) =>
    updateElement({ ...text, kind: "paragraph" });

  return <TextEditor label="Content" text={element} updateText={updateText} />;
}

type UpdateElement = (el: Element) => void;
type DeleteElement = (el: { id: string }) => void;

function EnumeratedElement({ n, kind }: { n: number; kind: Kind }) {
  const paddedNumber = n < 10 ? "0" + n : "" + n;
  const { label, icon: Icon } = kindMeta[kind];

  return (
    <HStack gap="2" color="colorPalette.fg">
      <Text fontFamily="mono" textStyle="sm" color="fg.muted">
        {paddedNumber}
      </Text>
      <Icon />
      <Text fontWeight="semibold">{label}</Text>
    </HStack>
  );
}

function BlockItem({ kind }: { kind: Kind }) {
  const { label, description, icon: Icon, palette } = kindMeta[kind];

  return (
    <Draggable
      id={kind}
      colorPalette={palette}
      display="flex"
      alignItems="center"
      gap="3"
      p="3"
      bg="colorPalette.subtle"
      borderWidth="1px"
      borderColor="colorPalette.muted"
      cursor="grab"
      userSelect="none"
      transition="all 0.15s"
      _hover={{
        borderColor: "colorPalette.solid",
        shadow: "md",
        transform: "translateY(-1px)",
      }}
      _active={{ cursor: "grabbing" }}
      css={{
        "&[data-dnd-dragging]": {
          cursor: "grabbing",
          shadow: "lg",
          borderColor: "colorPalette.solid",
        },
        "&[data-dnd-placeholder]": { opacity: 0.4, borderStyle: "dashed" },
      }}
    >
      <Center
        boxSize="10"
        flexShrink={0}
        bg="colorPalette.solid"
        color="colorPalette.contrast"
      >
        <Icon size="20" />
      </Center>
      <Box flex="1" minW="0">
        <Text fontWeight="semibold" color="colorPalette.fg">
          {label}
        </Text>
        <Text textStyle="xs" color="fg.muted">
          {description}
        </Text>
      </Box>
      <Box color="colorPalette.fg" opacity={0.6}>
        <LuGripVertical />
      </Box>
    </Draggable>
  );
}

function SectionHeading({ children }: { children: React.ReactNode }) {
  return (
    <Heading
      as="h2"
      textStyle="xs"
      fontWeight="semibold"
      textTransform="uppercase"
      color="fg.muted"
    >
      {children}
    </Heading>
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
    <VStack
      as="aside"
      align="stretch"
      gap="6"
      w="sm"
      flexShrink={0}
      bg="bg.subtle"
      borderStartWidth="1px"
      p="4"
      overflow="auto"
    >
      <VStack align="stretch" gap="2">
        <SectionHeading>Blocks</SectionHeading>
        {kinds.map((kind) => (
          <BlockItem key={kind} kind={kind} />
        ))}
      </VStack>
      <VStack align="stretch" gap="3">
        <SectionHeading>Page ({elements.length})</SectionHeading>
        {elements.length === 0 && (
          <Text textStyle="sm" color="fg.muted">
            No blocks yet.
          </Text>
        )}
        {elements.map((el, id) => (
          <Card.Root
            key={el.id}
            size="sm"
            colorPalette={kindMeta[el.kind].palette}
            borderStartWidth="4px"
            borderStartColor="colorPalette.solid"
          >
            <Card.Header>
              <Flex justifyContent={"space-between"} align="center">
                <EnumeratedElement n={id + 1} kind={el.kind} />
                <CloseButton size="sm" onClick={() => deleteElement(el)} />
              </Flex>
            </Card.Header>
            <Card.Body>
              {el.kind === "multiple-choice" ? (
                <MultipleChoiceEditor
                  element={el}
                  updateElement={updateElement}
                />
              ) : (
                <ParagraphEditor element={el} updateElement={updateElement} />
              )}
            </Card.Body>
          </Card.Root>
        ))}
      </VStack>
    </VStack>
  );
}

function Preview({ elements }: { elements: Element[] }) {
  if (elements.length === 0) {
    return (
      <Center
        flexDirection="column"
        gap="2"
        minH="60"
        borderWidth="2px"
        borderStyle="dashed"
        borderColor="colorPalette.muted"
        color="fg.muted"
      >
        <Box color="colorPalette.fg">
          <LuMousePointerClick size="28" />
        </Box>
        <Text>Drag blocks from the right.</Text>
      </Center>
    );
  }

  return (
    <VStack align="stretch" gap="4">
      {elements.map((el) =>
        el.kind == "paragraph" ? (
          <Text key={el.id}>{el.value}</Text>
        ) : (
          <MultipleChoice key={el.id} prompt={el.prompt} options={el.options} />
        ),
      )}
    </VStack>
  );
}

const paragraphElement = (id: string): Paragraph => ({
  id,
  value: "Placeholder paragraph",
  kind: "paragraph",
});
const multipleChoiceElement = (id: string): MultipleChoiceQuestion => ({
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
      <PageShell
        title="Question Builder"
        description="Author exams."
        path="/question-builder"
      >
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
          <Flex h="full">
            <Droppable
              id="droppable"
              flex="1"
              minW="0"
              p="8"
              overflow="auto"
              bg="bg.muted"
              transition="background 0.15s"
              css={{ "&[data-drop-target]": { bg: "colorPalette.subtle" } }}
            >
              <Box
                mx="auto"
                p="8"
                bg="bg"
                shadow="sm"
                borderWidth="1px"
                css={{
                  "[data-drop-target] &": { borderColor: "colorPalette.solid" },
                }}
              >
                <Preview elements={elements} />
              </Box>
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

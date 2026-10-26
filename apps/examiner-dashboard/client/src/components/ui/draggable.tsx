import { useDraggable } from "@dnd-kit/react";
import { Feedback } from "@dnd-kit/dom";
import { Box, type BoxProps } from "@chakra-ui/react";

export function Draggable({ id, ...props }: BoxProps & { id: string }) {
  const { ref } = useDraggable({
    id,
    plugins: [Feedback.configure({ feedback: "clone", dropAnimation: null })],
  });

  return <Box ref={ref} {...props} />;
}

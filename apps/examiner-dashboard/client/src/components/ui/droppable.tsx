import { Box, type BoxProps } from "@chakra-ui/react";
import { useDroppable } from "@dnd-kit/react";

export function Droppable({ id, ...props }: BoxProps & { id: string }) {
  const { ref, isDropTarget } = useDroppable({
    id,
  });

  return <Box ref={ref} data-drop-target={isDropTarget || undefined} {...props} />;
}

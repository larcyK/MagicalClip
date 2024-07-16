import { Component, createSignal } from 'solid-js';
import './ClipboardHistoryCard.css';
import { ClipboardData } from './bindings';
import { 
  HStack, 
  Container,
  Center,
} from '@hope-ui/solid';

function parseRFC3339(dateString: string): Date {
  const date = new Date(dateString);
  if (isNaN(date.getTime())) {
      throw new Error("Invalid RFC3339 date string");
  }
  return date;
}

function formatDate(date: Date): string {
  const year = date.getUTCFullYear();
  const month = (date.getUTCMonth() + 1).toString().padStart(2, '0');
  const day = date.getUTCDate().toString().padStart(2, '0');
  const hours = date.getUTCHours().toString().padStart(2, '0');
  const minutes = date.getUTCMinutes().toString().padStart(2, '0');
  const seconds = date.getUTCSeconds().toString().padStart(2, '0');
  const milliseconds = date.getUTCMilliseconds().toString().padStart(3, '0');

  return `${year}-${month}-${day}\n${hours}:${minutes}:${seconds}.${milliseconds}`;
}

function formatJSTDate(date: Date): string {
  const jstOffset = 9 * 60;
  const jstDate = new Date(date.getTime() + jstOffset * 60 * 1000);

  const year = jstDate.getUTCFullYear();
  const month = (jstDate.getUTCMonth() + 1).toString().padStart(2, '0');
  const day = jstDate.getUTCDate().toString().padStart(2, '0');
  const hours = jstDate.getUTCHours().toString().padStart(2, '0');
  const minutes = jstDate.getUTCMinutes().toString().padStart(2, '0');
  const seconds = jstDate.getUTCSeconds().toString().padStart(2, '0');
  const milliseconds = jstDate.getUTCMilliseconds().toString().padStart(3, '0');

  return `${year}-${month}-${day}\n${hours}:${minutes}:${seconds}.${milliseconds}`;
}

interface ClipboardHistoryCardProps {
  clipboardData: ClipboardData;
  onCopy: () => void;
  onDelete: () => void;
}

const ClipboardHistoryCard: Component<ClipboardHistoryCardProps> = (props) => {
  const date = parseRFC3339(props.clipboardData.datetime);
  const formattedDate = formatJSTDate(date);
  const [isCopied, setIsCopied] = createSignal(false);

  function onCopy() {
    props.onCopy();
    setIsCopied(true);
    setTimeout(() => setIsCopied(false), 5000);
  }

  return (
    <HStack width="100%" spacing="$4" padding="$4"  class="clipboard-history-card">
      <Center height="50px" fontSize="14px">
        {formattedDate}
      </Center>
      <Container class="card-content" style={{"text-align": "left"}}>
        {props.clipboardData.data}
      </Container>
      <button onClick={onCopy} class="common-button" disabled={isCopied()}>
        {isCopied() ? "✅" : "📋"}
      </button>
      <button onClick={props.onDelete} class="common-button">🗑️</button>
    </HStack>
  );
};

export default ClipboardHistoryCard;

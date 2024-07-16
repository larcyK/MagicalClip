import { Component } from 'solid-js';
import './ClipboardHistoryCard.css';
import { ClipboardData } from './bindings';
import { 
  HStack, 
  Container,
  Center,
  Divider,
  HopeProvider,
  Text,
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

interface ClipboardHistoryCardProps {
  clipboardData: ClipboardData;
  onCopy: () => void;
  onDelete: () => void;
}

const ClipboardHistoryCard: Component<ClipboardHistoryCardProps> = (props) => {
  const date = parseRFC3339(props.clipboardData.datetime);
  const formattedDate = formatDate(date);

  return (
    <HStack width="100%" spacing="$4" padding="$4"  class="clipboard-history-card">
      <Center height="50px" fontSize="12px">
        {formattedDate}
      </Center>
      <Container class="card-content" style={{"text-align": "left"}}>
        {props.clipboardData.data}
      </Container>
      <button onClick={props.onCopy} class="common-button">📋</button>
      <button onClick={props.onDelete} class="common-button">🗑️</button>
    </HStack>
  );
};

export default ClipboardHistoryCard;

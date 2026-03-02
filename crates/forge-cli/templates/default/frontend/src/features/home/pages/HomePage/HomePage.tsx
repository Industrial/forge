import { Heading, Text } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

export default function Home() {
  return (
    <>
      <Heading level={1} styles={style({ font: 'heading-xl' })}>
        Home
      </Heading>
      <Text>Welcome. You are logged in.</Text>
    </>
  );
}

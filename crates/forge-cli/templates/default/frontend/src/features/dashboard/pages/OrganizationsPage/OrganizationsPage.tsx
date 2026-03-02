import { Heading, Text } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

export default function Organizations() {
  return (
    <>
      <Heading level={1} styles={style({ font: 'heading-xl' })}>
        Organizations
      </Heading>
      <Text>Manage organizations.</Text>
    </>
  );
}

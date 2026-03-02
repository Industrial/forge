import { Button, Heading, Text } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import { useSession } from '../../../../context/Session';

export default function ProfilePage() {
  const { user } = useSession();

  return (
    <>
      <Heading level={1} styles={style({ font: 'heading-xl' })}>
        Profile
      </Heading>
      {user != null && (
        <div className={style({ display: 'flex', flexDirection: 'column', gap: 8 })}>
          <Text>Email: {user.email}</Text>
          <form action="/api/auth/logout" method="get" target="_top">
            <Button type="submit" variant="secondary">
              Log out
            </Button>
          </form>
        </div>
      )}
    </>
  );
}

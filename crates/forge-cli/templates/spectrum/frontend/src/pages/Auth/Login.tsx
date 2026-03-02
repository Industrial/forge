import { Button, Form, Heading, Link, TextField } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

export default function Login() {
  return (
    <>
      <Heading level={1} styles={style({ font: 'heading-xl' })}>Log in</Heading>
      <div className={style({ display: 'flex', flexDirection: 'column', gap: 8 })}>
        <Form
          action="/api/auth/login"
          method="post"
          target="_top"
        >
          <TextField
            name="email"
            type="email"
            label="Email"
            placeholder="you@example.com"
            isRequired
          />
          <TextField
            name="password"
            type="password"
            label="Password"
            placeholder="••••••••"
            isRequired
          />
          <div
            className={style({
              display: 'flex',
              flexDirection: 'row',
              justifyContent: 'end',
            })}
          >
            <Button type="submit" variant="accent">
              Log in
            </Button>
          </div>
        </Form>
        <div className={style({ textAlign: 'center', font: 'body' })}>
          <Link href="/register">Don&apos;t have an account? Create an Account</Link>
        </div>
      </div>
    </>
  );
}

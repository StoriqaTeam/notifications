UPDATE templates 
SET data = '<html>
  <head>
    <title>Successful password reset on Storiqa</title>
  </head>
  <body>
    <p>
       Dear {{user.first_name}},
      <br/>

        Congratulations! Your password has been changed successfully!
      <br/>
        <a href="http://storiqa.com">Let&apos;s go Storiqa!</a>
      <br/>
        This is an automatically generated email – please do not reply to it.
      <br/>

    </p>

  </body>
</html>'
WHERE
name = 'apply_password_reset_for_user';
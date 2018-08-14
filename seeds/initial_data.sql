DELETE FROM templates;

ALTER SEQUENCE templates_id_seq RESTART WITH 1;

INSERT INTO templates(name, data ) VALUES
('store_order_create', '<html>
  <head>
    <title>New order {{order_slug}}</title>
  </head>
  <body>
    <p>
      Please be informed that you have a new order {{order_slug}}. 
      <br/>
      You can watch your order on <a href= &quot;{{cluster_url}}/manage/store/{{store_id}}/orders/{{order_slug}}&quot;>this page</a> .
      <br/>
      Best regards,
      Storiqa Team
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>'),
('store_order_update', '<html>
  <head>
    <title>The order {{order_slug}} status</title>
  </head>
  <body>
    <p>
      Please be informed that the order {{order_slug}} status has changed to {{order_state}}.
      <br/>
      You can watch your order on <a href=&quot;{{cluster_url}}/manage/store/{{store_id}}/orders/{{order_slug}}&quot;>this page</a> .
      <br/>
      Best regards,
      Storiqa Team
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>
'),
('user_email_verification_apply', '<html>
  <head>
    <title>Successful registration on Storiqa</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      Your e-mail address is successfully confirmed and registration process is completely finished. Thank you for joining us!
      <br/>
      Best regards,
      Storiqa Team  
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>
'),
('user_email_verification', '<html>
  <head>
    <title>Verify your account on Storiqa</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      Thank you for signing up for Storiqa! In order to finish the registration process and verify your account, please confirm your e-mail by following the link below:
      
     <a href=&quot;{{verify_email_path}}/{{token}}&quot;>Verify my email on Storiqa.</a>
      <br/>

      Best regards,
      Storiqa Team
      <br/>
      
      Note: If you didn&apos;t initiate the request, please delete this e-mail. 
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>
    </p>

  </body>
</html>
'),
('user_order_create', '<html>
  <head>
    <title>New order {{order_slug}}</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      Please be informed that you have a new order {{order_slug}}. 
      <br/>
      You can watch your order on <a href=&quot;{{cluster_url}}/profile/orders/{{order_slug}}&quot;>this page</a>.
      <br/>
      Best regards,
      Storiqa Team
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>
'),
('user_order_update', '<html>
  <head>
    <title>The order {{order_slug}} status</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      Please be informed that the order {{order_slug}} status has changed to {{order_state}}.
      <br/>
      You can watch your order on <a href=&quot;{{cluster_url}}/profile/orders/{{order_slug}}&quot;>this page</a>.
      <br/>
      Best regards,
      Storiqa Team
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>
'),
('user_reset_password_apply', '<html>
  <head>
    <title>Successful password reset on Storiqa</title>
  </head>
  <body>
    <p>
       Dear {{user.first_name}},
      <br/>

        Congratulations! Your password has been changed successfully!
      <br/>
        <a href=&quot;http://storiqa.com&quot;>Let&apos;s go Storiqa!</a>
      <br/>
        This is an automatically generated email – please do not reply to it.
      <br/>

    </p>

  </body>
</html>
'),
('user_reset_password', '<html>
  <head>
    <title>Password reset on Storiqa</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      
      You received this e-mail because you have made a request to change your password. In order to do that, please follow the link below:
      
     <a href=&quot;{{reset_password_path}}/{{token}}&quot;>Reset my password on Storiqa.</a>
      <br/>
      
      Best regards,
      Storiqa Team
      <br/>
      
      Note: If you have received a password-reset email you didn&apos;t request, it&apos;s likely that someone entered your e-mail address by mistake. If you didn&apos;t initiate this request, please delete this e-mail. Your privacy and security aren&apos;t compromised by this e-mail.
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>
');
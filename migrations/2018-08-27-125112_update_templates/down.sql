UPDATE templates 
SET data = '<html>
  <head>
    <title>The order {{order_slug}} status</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      Please be informed that the order {{order_slug}} status has changed to {{order_state}}.
      <br/>
      You can watch your order on <a href="{{cluster_url}}/profile/orders/{{order_slug}}">this page</a>.
      <br/>
      Best regards,
      Storiqa Team
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>'
WHERE
name = 'order_update_state_for_user';